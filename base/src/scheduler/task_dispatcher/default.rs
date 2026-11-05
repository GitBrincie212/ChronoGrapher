use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::{ErasedTaskHandleAlias, TaskHandleExt};
use dashmap::DashMap;
use tokio::sync::Notify;

pub struct DefaultTaskDispatcher<C: SchedulerConfig>(DashMap<ErasedTaskHandleAlias<C>, Notify>);

impl<C: SchedulerConfig> Default for DefaultTaskDispatcher<C> {
    fn default() -> Self {
        Self(DashMap::new())
    }
}

impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher<C> {
    fn dispatch(
        &self,
        handle: &ErasedTaskHandleAlias<C>,
    ) -> impl Future<Output = Result<(), C::TaskError>> + Send {
        // TODO: Find a way to remove the Notify when a Task is removed
        let notifier = self
            .0
            .entry(handle.clone())
            .or_insert_with(|| Notify::new());

        async move {
            tokio::select! {
                result = handle.run() => result,
                _ = notifier.notified() => Ok(()),
            }
        }
    }

    fn cancel(&self, id: &ErasedTaskHandleAlias<C>) -> impl Future<Output = ()> + Send {
        if let Some((_, tok)) = self.0.remove(id) {
            tok.notify_one()
        }
        std::future::ready(())
    }
}
