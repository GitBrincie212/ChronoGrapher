pub mod main_loop;
pub mod scheduler_handle;
pub mod rescheduling;

use std::sync::Arc;
pub use main_loop::*;
pub use scheduler_handle::*;
pub use rescheduling::*;
use crate::prelude::SchedulerConfig;
use crate::scheduler::{TriggerJobWorkers, TRIGGER_WORKER_POOL};
use crate::task::TaskTrigger;
use crate::utils::TaskIdentifier;

pub async fn assign_to_trigger_worker<C: SchedulerConfig>(
    trigger: Arc<dyn TaskTrigger>,
    id: &C::TaskIdentifier,
    trigger_workers: &TriggerJobWorkers<C>
) {
    let idx = id.as_usize() & (TRIGGER_WORKER_POOL - 1);
    trigger_workers[idx]
        .send((id.clone(), trigger))
        .await
        .expect("Could not send Task to TaskTrigger workers");
}