use std::any::{type_name, Any};
use std::sync::Arc;
use crossbeam::queue::SegQueue;
use tokio::sync::Notify;
use crate::scheduler::{assign_to_trigger_worker, spawn_task, SchedulerConfig, SchedulerHandlePayload, SchedulerWorker};
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ErasedTask, TaskHook};

pub enum SchedulerHandleInstructions {
    Reschedule, // Forces the Task to reschedule (instances may still run)
    Halt,       // Cancels the Task's current execution, if any
    Block,      // Blocks the Task from rescheduling
    Execute,    // Spawns a new instance of the Task to run
}

pub struct SchedulerHandle {
    pub id: Arc<dyn Any + Send + Sync>,
    pub channel: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
}

impl SchedulerHandle {
    pub(crate) async fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel.0.push((self.id.clone(), instruction));
        self.channel.1.notify_waiters();
    }
}

impl TaskHook<()> for SchedulerHandle {}

pub async fn append_scheduler_handler<C: SchedulerConfig>(
    task: &ErasedTask<C::TaskError>,
    id: C::TaskIdentifier,
    channel: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
) {
    let handle = SchedulerHandle {
        id: Arc::new(id),
        channel,
    };

    task.attach_hook::<()>(Arc::new(handle)).await;
}

#[inline(always)]
pub fn scheduler_handle_instructions_logic<C: SchedulerConfig>(
    instruct_queue: &Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
    dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    store: &Arc<C::SchedulerTaskStore>,
    workers: &Arc<Vec<SchedulerWorker<C>>>,
) -> impl Future<Output = ()> + 'static {
    let dispatcher = dispatcher.clone();
    let store = store.clone();
    let workers = workers.clone();
    let instruct_queue = instruct_queue.clone();

    async move {
        while let Some((id, instruction)) = instruct_queue.0.pop() {
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
                    dispatcher.cancel(&id).await;
                }

                SchedulerHandleInstructions::Block => {
                    store.remove(&id);
                }

                SchedulerHandleInstructions::Execute => {
                    spawn_task::<C>(id.clone(), workers.as_ref());
                }
            }
        }

        instruct_queue.1.notified().await;
    }
}