pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::prelude::SchedulerConfig;
use crate::scheduler::SchedulerWorker;
use crate::utils::TaskIdentifier;

pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: C::TaskIdentifier,
    workers: &Vec<SchedulerWorker<C>>
) {
    let idx = id.as_usize() & (workers.len() - 1);
    workers[idx].trigger_queue.push(id);
    workers[idx].notify.notify_waiters();
}