pub mod main_loop;
pub mod scheduler_handle;

pub use main_loop::*;
pub use scheduler_handle::*;
use crate::scheduler::{SchedulerConfig, SchedulerKey, SchedulerWorker};

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: SchedulerKey<C>,
    workers: &[SchedulerWorker<C>]
) {
    let idx = id.clone().into() & (workers.len() - 1);
    workers[idx].spawn_trigger(id);
}