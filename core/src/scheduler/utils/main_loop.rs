use std::sync::Arc;
use crate::prelude::SchedulerConfig;
use crate::scheduler::{spawn_task, Worker};
use crate::scheduler::engine::SchedulerEngine;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    dispatch_workers: &Arc<Vec<Worker<C::TaskIdentifier>>>
) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();
    let dispatch_workers = dispatch_workers.clone();

    async move {
        loop {
            for id in engine.retrieve().await {
                spawn_task::<C>(id, dispatch_workers.clone());
            }
        }
    }
}