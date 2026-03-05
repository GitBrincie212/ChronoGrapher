use std::sync::Arc;
use crate::prelude::SchedulerConfig;
use crate::scheduler::{spawn_task, ReschedulePayload};
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::task_store::SchedulerTaskStore;

#[inline(always)]
pub fn main_loop_logic<C: SchedulerConfig>(
    engine: &Arc<C::SchedulerEngine>,
    dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    store: &Arc<C::SchedulerTaskStore>,
    scheduler_send: tokio::sync::mpsc::Sender<ReschedulePayload<C>>,
) -> impl Future<Output = ()> + 'static {
    let engine = engine.clone();
    let dispatcher = dispatcher.clone();
    let store = store.clone();

    async move {
        loop {
            for id in engine.retrieve().await {
                if let Some(task) = store.get(&id) {
                    spawn_task::<C>(id.clone(), scheduler_send.clone(), &dispatcher, task);
                }
            }
        }
    }
}