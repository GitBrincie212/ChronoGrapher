use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::ops::Deref;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;

pub struct DefaultTaskDispatcher<C: SchedulerConfig>(
    DashMap<C::TaskIdentifier, CancellationToken>
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
        let tok = CancellationToken::new();
        self.0.insert(id.clone(), tok.clone());

        tokio::select! {
            result = task.run() => {
                self.0.remove(id);
                return result;
            }

            _ = tok.cancelled() => Ok(()),
        }

    }

    async fn cancel(&self, id: &C::TaskIdentifier) {
        if let Some((_, tok)) = self.0.remove(id) {
            tok.cancel()
        }
    }
}
