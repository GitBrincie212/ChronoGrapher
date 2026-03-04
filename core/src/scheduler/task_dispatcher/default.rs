use crate::scheduler::Arc;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::{EngineNotifier, SchedulerTaskDispatcher};
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::ops::Deref;
use tokio::task::JoinHandle;
use dashmap::DashMap;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;

pub struct DefaultTaskDispatcher<C: SchedulerConfig>(
    Arc<DashMap<C::TaskIdentifier, Vec<JoinHandle<()>>>>
);

impl<C: SchedulerConfig> Default for DefaultTaskDispatcher<C> {
    fn default() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher<C> {
    async fn dispatch(
        &self,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
        engine_notifier: &EngineNotifier<C>,
    ) {
        let engine_id = engine_notifier.id().clone();
        let dispatcher = self.0.clone();
        let notifier = engine_notifier.clone();

        let mut entry = self.0
            .entry(engine_id.clone())
            .or_default();

        let handle = tokio::spawn(async move {
            let result = task.deref().run().await;

            notifier.notify(result.err()).await;

            if let Some(mut entry) = dispatcher.get_mut(&engine_id) {
                entry.retain(|h| !h.is_finished());
                if entry.is_empty() {
                    drop(entry);
                    dispatcher.remove(&engine_id);
                }
            }
        });

        entry.push(handle);
    }

    async fn cancel(&self, id: &C::TaskIdentifier) {
        if let Some((_, notifiers)) = self.0.remove(id) {
            for tx in notifiers.into_iter() {
                let _ = tx.abort();
            }
        }
    }
}
