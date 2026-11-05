use crate::scheduler::impls::utils::{assign_to_trigger_worker, spawn_task};
use crate::scheduler::live::SchedulerWorkerHot;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::{BoundedTaskHandle, ErasedTaskHandleAlias};
use crate::scheduler::{
    LiveSchedulerComponents, SchedulerConfig, SchedulerHandlePayload, SchedulerWorkerCold,
};
use crate::storage::TaskHandle;
use crossbeam::queue::SegQueue;
use crossbeam::utils::CachePadded;
use std::any::{Any, type_name};
use std::sync::Arc;
use tokio::sync::Notify;

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
    pub(crate) fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel.0.push((self.id.clone(), instruction));
        self.channel.1.notify_waiters();
    }
}

#[inline(always)]
pub fn append_scheduler_handler<C: SchedulerConfig>(
    task_handle: ErasedTaskHandleAlias<C>,
    channel: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
) -> impl Future<Output = ()> + Send {
    let handle = SchedulerHandle {
        id: Arc::new(task_handle.clone()),
        channel,
    };

    async move {
        let _ = task_handle.attach_hook(handle).await;
    }
}

#[inline(always)]
pub fn scheduler_handle_instructions_logic<C: SchedulerConfig>(
    instruct_queue: &Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
    components: &Arc<LiveSchedulerComponents<C>>,
    hot_workers: &Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: &Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
) -> impl Future<Output = ()> + Send + 'static {
    let components = components.clone();
    let hot_workers = hot_workers.clone();
    let cold_workers = cold_workers.clone();
    let instruct_queue = instruct_queue.clone();

    async move {
        while let Some((handle, instruction)) = instruct_queue.0.pop() {
            let handle = handle
                .downcast_ref::<ErasedTaskHandleAlias<C>>()
                .unwrap_or_else(|| {
                    panic!(
                        "Cannot downcast to TaskIdentifier of type {:?}",
                        type_name::<ErasedTaskHandleAlias<C>>()
                    )
                });

            match instruction {
                SchedulerHandleInstructions::Reschedule => {
                    assign_to_trigger_worker::<C>(handle.clone(), &hot_workers, &cold_workers);
                }

                SchedulerHandleInstructions::Halt => {
                    components.dispatcher.cancel(handle).await;
                }

                SchedulerHandleInstructions::Block => {
                    handle.invalidate().await;
                }

                SchedulerHandleInstructions::Execute => {
                    spawn_task::<C>(handle.clone(), &hot_workers, &cold_workers);
                }
            }
        }

        instruct_queue.1.notified().await;
    }
}
