use std::sync::Arc;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::engine::SchedulerEngine;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(engine: &Arc<C::SchedulerEngine>) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();

    async move {
        loop {
            for handle in engine.retrieve().await {
                handle.dispatch();
            }
        }
    }
}