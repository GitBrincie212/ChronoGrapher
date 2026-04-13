pub mod main_loop;
pub mod scheduler_handle;

pub use main_loop::*;
pub use scheduler_handle::*;

use crate::scheduler::{SchedulerConfig, SchedulerKey};
use crate::scheduler::impls::live::SchedulerWorker;

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: SchedulerKey<C>,
    workers: &[SchedulerWorker<C>]
) {
    let idx = id.clone().into() & (workers.len() - 1);
    workers[idx].spawn_trigger(id);
}

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(
    key: SchedulerKey<C>,
    dispatch_workers: &[SchedulerWorker<C>]
) {
    let idx = key.clone().into() & (dispatch_workers.len() - 1);
    dispatch_workers[idx].spawn_dispatch(key);
}