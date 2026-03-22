use std::any::{type_name, Any};
use std::sync::Arc;
use crossbeam::queue::SegQueue;
use tokio::sync::Notify;
use crate::scheduler::{assign_dispatching_to_worker, assign_triggering_to_worker, SchedulerConfig, SchedulerHandlePayload, SchedulerWorker};
use crate::task::{ErasedTask, TaskHandle, TaskHook, TaskRef};

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

#[inline(always)]
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
    workers: &Arc<Vec<SchedulerWorker<C>>>,
) -> impl Future<Output = ()> + 'static {
    let workers = workers.clone();
    let instruct_queue = instruct_queue.clone();

    async move {
        while let Some((handle, instruction)) = instruct_queue.0.pop() {
            let handle = handle.downcast_ref::<TaskHandle<C>>().unwrap_or_else(|| {
                panic!(
                    "Cannot downcast to TaskIdentifier of type {:?}",
                    type_name::<C::TaskIdentifier>()
                )
            });

            match instruction {
                SchedulerHandleInstructions::Reschedule => {
                    assign_triggering_to_worker::<C>(handle.clone(), workers.as_ref());
                }

                SchedulerHandleInstructions::Halt => {
                    handle.cancel().await;
                }

                SchedulerHandleInstructions::Block => {
                    handle.remove().await;
                }

                SchedulerHandleInstructions::Execute => {
                    assign_dispatching_to_worker::<C>(handle.clone(), workers.as_ref());
                }
            }
        }

        instruct_queue.1.notified().await;
    }
}