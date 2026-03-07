use std::any::{type_name, Any};
use std::sync::Arc;
use crate::prelude::{SchedulerConfig, TaskHook};
use crate::scheduler::{assign_to_trigger_worker, spawn_task, SchedulerHandlePayload, SchedulerWorker};
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::ErasedTask;

pub enum SchedulerHandleInstructions {
    Reschedule, // Forces the Task to reschedule (instances may still run)
    Halt,       // Cancels the Task's current execution, if any
    Block,      // Blocks the Task from rescheduling
    Execute,    // Spawns a new instance of the Task to run
}

pub struct SchedulerHandle {
    pub id: Arc<dyn Any + Send + Sync>,
    pub channel: tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
}

impl SchedulerHandle {
    pub(crate) async fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel
            .send((self.id.clone(), instruction))
            .await
            .expect("Cannot instruct");
    }
}

impl TaskHook<()> for SchedulerHandle {}

pub async fn append_scheduler_handler<C: SchedulerConfig>(
    task: &ErasedTask<C::TaskError>,
    id: C::TaskIdentifier,
    channel: tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
) {
    let handle = SchedulerHandle {
        id: Arc::new(id),
        channel,
    };

    task.attach_hook::<()>(Arc::new(handle)).await;
}

#[inline(always)]
pub fn scheduler_handle_instructions_logic<C: SchedulerConfig>(
    mut instruct_receive: tokio::sync::mpsc::Receiver<SchedulerHandlePayload>,
    dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    store: &Arc<C::SchedulerTaskStore>,
    workers: &Arc<Vec<SchedulerWorker<C>>>,
) -> impl Future<Output = ()> + 'static {
    let dispatcher = dispatcher.clone();
    let store = store.clone();
    let workers = workers.clone();

    async move {
        while let Some((id, instruction)) = instruct_receive.recv().await {
            let id = id.downcast_ref::<C::TaskIdentifier>().unwrap_or_else(|| {
                panic!(
                    "Cannot downcast to TaskIdentifier of type {:?}",
                    type_name::<C::TaskIdentifier>()
                )
            });

            match instruction {
                SchedulerHandleInstructions::Reschedule => {
                    assign_to_trigger_worker::<C>(id.clone(), workers.as_ref());
                }

                SchedulerHandleInstructions::Halt => {
                    dispatcher.cancel(id).await;
                }

                SchedulerHandleInstructions::Block => {
                    store.remove(id);
                }

                SchedulerHandleInstructions::Execute => {
                    spawn_task::<C>(id.clone(), workers.as_ref());
                }
            }
        }
    }
}