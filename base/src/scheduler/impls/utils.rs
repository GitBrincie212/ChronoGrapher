pub mod main_loop;
pub mod scheduler_handle;

pub use main_loop::*;
pub use scheduler_handle::*;

use crate::scheduler::impls::live::{SchedulerWork, SchedulerWorker};
use crate::scheduler::{SchedulerConfig, SchedulerKey};
use crossbeam::deque::Injector;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: SchedulerKey<C>,
    workers: &Arc<Vec<SchedulerWorker<C>>>,
) {
    let idx = fastrand::usize(..workers.len());
    workers[idx].ingress.push((id, SchedulerWork::Trigger));
    workers[idx].notify.notify_one();
}

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(key: SchedulerKey<C>, workers: &Arc<Vec<SchedulerWorker<C>>>) {
    let idx = fastrand::usize(..workers.len());
    workers[idx].ingress.push((key, SchedulerWork::Dispatch));
    workers[idx].notify.notify_one();
}
