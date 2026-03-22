pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::scheduler::{SchedulerConfig, SchedulerWorker};
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::TaskHandle;
use crate::utils::TaskIdentifier;

#[inline(always)]
pub fn assign_triggering_to_worker<C: SchedulerConfig>(
    handle: TaskHandle<C>,
    workers: &Vec<SchedulerWorker<C>>
) {
    let idx = fastrand::usize(0..workers.len());
    workers[idx].spawn_trigger(handle);
}

#[inline(always)]
pub fn assign_dispatching_to_worker<C: SchedulerConfig>(
    handle: TaskHandle<C>,
    workers: &Vec<SchedulerWorker<C>>
) {
    let idx = fastrand::usize(0..workers.len());
    workers[idx].spawn_trigger(handle);
}