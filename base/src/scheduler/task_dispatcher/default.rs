use crate::scheduler::Arc;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::ops::Deref;
use dashmap::DashMap;
use tokio::sync::Notify;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;

pub struct DefaultTaskDispatcher<C: SchedulerConfig>(
    DashMap<C::TaskIdentifier, Vec<Arc<Notify>>>
);

impl<C: SchedulerConfig> Default for DefaultTaskDispatcher<C> {
    fn default() -> Self {
        Self(DashMap::new())
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher<C> {
    async fn dispatch(
        &self,
        id: &C::TaskIdentifier,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
    ) -> Result<(), C::TaskError> {
        let mut entry = self.0
            .entry(id.clone())
            .or_default();

        let handle = Arc::new(Notify::new());
        entry.push(handle.clone());
        drop(entry);

        tokio::select! {
            result = task.run() => {
                self.0.remove(id);
                return result;
            }

            _ = handle.notified() => Ok(()),
        }

    }

    async fn cancel(&self, id: &C::TaskIdentifier) {
        if let Some((_, notifiers)) = self.0.remove(id) {
            for handle in notifiers.into_iter() {
                handle.notify_waiters();
            }
        }
    }
}
