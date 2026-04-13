use std::sync::Arc;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::impls::live::SchedulerWorker;
use crate::scheduler::impls::utils::spawn_task;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    workers: &Arc<Vec<SchedulerWorker<C>>>
) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();
    let workers = workers.clone();

    async move {
        loop {
            for id in engine.retrieve().await {
                spawn_task::<C>(id, workers.as_ref());
            }
        }
    }
}