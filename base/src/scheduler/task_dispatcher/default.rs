use crate::scheduler::{SchedulerConfig, SchedulerKey};
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::ErasedTask;
use std::ops::Deref;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::Notify;

pub struct DefaultTaskDispatcher<C: SchedulerConfig>(
    DashMap<SchedulerKey<C>, Arc<Notify>>
);

impl<C: SchedulerConfig> Default for DefaultTaskDispatcher<C> {
    fn default() -> Self {
        Self(DashMap::new())
    }
}

impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher<C> {
    fn dispatch(
        &self,
        key: &SchedulerKey<C>,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
    ) -> impl Future<Output = Result<(), C::TaskError>> + Send {
        
        // TODO: Find a way to remove the Notify when a Task is removed
        let notifier = self.0
            .entry(key.clone())
            .or_insert_with(|| Arc::new(Notify::new()));

        async move {
            tokio::select! {
                result = task.run() => result,
                _ = notifier.notified() => Ok(()),
            }
        }
    }

    fn cancel(&self, id: &SchedulerKey<C>) -> impl Future<Output = ()> + Send {
        if let Some((_, tok)) = self.0.remove(id) {
            tok.notify_one()
        }
        std::future::ready(())
    }
}