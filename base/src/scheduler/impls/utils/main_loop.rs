use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::impls::live::SchedulerWorkerHot;
use crate::scheduler::impls::utils::spawn_task;
use crate::scheduler::{SchedulerConfig, SchedulerWorkerCold};
use crossbeam::utils::CachePadded;
use std::sync::Arc;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    hot_workers: &Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: &Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();
    let hot_workers = hot_workers.clone();
    let cold_workers = cold_workers.clone();

    async move {
        loop {
            for id in engine.retrieve().await {
                spawn_task::<C>(id, &hot_workers, &cold_workers);
            }
        }
    }
}
