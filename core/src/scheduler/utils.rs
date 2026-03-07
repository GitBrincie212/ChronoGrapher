pub mod main_loop;
pub mod rescheduling;
pub mod scheduler_handle;

use crate::prelude::SchedulerConfig;
use crate::scheduler::SchedulerWorker;
use crate::task::TaskTrigger;
use crate::utils::TaskIdentifier;
pub use main_loop::*;
pub use rescheduling::*;
pub use scheduler_handle::*;
use std::sync::Arc;

pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    trigger: Arc<dyn TaskTrigger>,
    id: &C::TaskIdentifier,
    workers: &Vec<SchedulerWorker<C>>,
) {
    let idx = id.as_usize() & (workers.len() - 1);
    workers[idx].trigger_queue.push((id.clone(), trigger));
    workers[idx].notify.notify_waiters();
}
