pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

use std::sync::Arc;
pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::prelude::SchedulerConfig;
use crate::scheduler::TriggerJobWorker;
use crate::task::TaskTrigger;
use crate::utils::TaskIdentifier;

pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    trigger: Arc<dyn TaskTrigger>,
    id: &C::TaskIdentifier,
    trigger_workers: &Vec<TriggerJobWorker<C>>
) {
    let idx = id.as_usize() & (trigger_workers.len() - 1);
    trigger_workers[idx].0.push((id.clone(), trigger));
    trigger_workers[idx].1.notify_waiters();
}