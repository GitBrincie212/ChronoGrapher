use crate::prelude::SchedulerConfig;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::{SchedulerWorker, spawn_task};
use std::sync::Arc;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    workers: &Arc<Vec<SchedulerWorker<C>>>,
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
