pub mod main_loop;
pub mod scheduler_handle;

pub use main_loop::*;
pub use scheduler_handle::*;

use crate::scheduler::impls::live::{SchedulerWork, SchedulerWorkerHot};
use crate::scheduler::{SchedulerConfig, SchedulerKey, SchedulerWorkerCold};
use crossbeam::utils::CachePadded;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[inline(always)]
pub fn assign_to_trigger_worker<C: SchedulerConfig>(
    key: SchedulerKey<C>,
    hot_workers: &Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: &Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
) {
    let idx = fastrand::usize(..hot_workers.len());
    hot_workers[idx].ingress.push((key, SchedulerWork::Trigger));
    let prev = cold_workers[idx].pending.fetch_add(1, Ordering::Relaxed);
    if prev == 0 {
        cold_workers[idx].notify.notify_one();
    }
}

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(
    key: SchedulerKey<C>,
    hot_workers: &Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: &Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
) {
    let idx = fastrand::usize(..hot_workers.len());
    hot_workers[idx]
        .ingress
        .push((key, SchedulerWork::Dispatch));
    let prev = cold_workers[idx].pending.fetch_add(1, Ordering::Relaxed);
    if prev == 0 {
        cold_workers[idx].notify.notify_one();
    }
}
