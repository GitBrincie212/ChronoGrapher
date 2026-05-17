pub mod main_loop;
pub mod scheduler_handle;

pub use main_loop::*;
pub use scheduler_handle::*;

use crate::scheduler::impls::live::{SchedulerWork, SchedulerWorker};
use crate::scheduler::{SchedulerConfig, SchedulerKey};
use std::sync::Arc;
use crossbeam::utils::CachePadded;

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    id: SchedulerKey<C>,
    workers: &Arc<Vec<CachePadded<SchedulerWorker<C>>>>,
) {
    let idx = fastrand::usize(..workers.len());
    workers[idx].ingress.push((id, SchedulerWork::Trigger));
    /*
    let prev = workers[idx].pending.fetch_add(1, Ordering::Relaxed);
    if prev == 0 {
        workers[idx].notify.notify_one();
    }
     */
    workers[idx].notify.notify_one();
}

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(key: SchedulerKey<C>, workers: &Arc<Vec<CachePadded<SchedulerWorker<C>>>>) {
    let idx = fastrand::usize(..workers.len());
    workers[idx].ingress.push((key, SchedulerWork::Dispatch));
    /*
    let prev = workers[idx].pending.fetch_add(1, Ordering::Relaxed);
    if prev == 0 {
        workers[idx].notify.notify_one();
    }
     */
    workers[idx].notify.notify_one();
}
