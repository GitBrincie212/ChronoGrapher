pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::scheduler::{SchedulerConfig, WorkerPool};
use crate::utils::TaskIdentifier;

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: C::TaskIdentifier,
    pool: &WorkerPool<C>,
    workers: usize,
) {
    let _ = id.as_usize() & (workers.saturating_sub(1));
    pool.spawn_trigger(id);
}