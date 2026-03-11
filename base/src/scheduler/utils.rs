pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::scheduler::{SchedulerConfig, SchedulerWorker};
use crate::utils::TaskIdentifier;

pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: C::TaskIdentifier,
    workers: &Vec<SchedulerWorker<C>>
) {
    let idx = id.as_usize() & (workers.len() - 1);
    workers[idx].spawn_trigger(id);
}