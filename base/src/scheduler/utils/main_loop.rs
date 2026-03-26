use std::sync::Arc;
use crate::scheduler::{spawn_task, SchedulerConfig, WorkerPool};
use crate::scheduler::engine::SchedulerEngine;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    pool: &Arc<WorkerPool<C>>,
    workers: usize,
) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();
    let pool = pool.clone();

    async move {
        loop {
            for id in engine.retrieve().await {
                spawn_task::<C>(id, pool.as_ref(), workers);
            }
        }
    }
}