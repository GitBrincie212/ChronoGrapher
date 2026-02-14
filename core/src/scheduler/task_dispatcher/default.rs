use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::{EngineNotifier, SchedulerTaskDispatcher};
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;

#[derive(Default, Clone, Copy)]
pub struct DefaultTaskDispatcher;

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher {
    async fn dispatch(&self, task: Arc<ErasedTask<C::Error>>, notifier: EngineNotifier<C>) {
        let res = task.run().await;
        notifier.notify(res.err()).await;
    }
}
