use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::{TaskHandle, TaskRef};
use async_trait::async_trait;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;

#[derive(Default, Copy, Clone)]
pub struct DefaultTaskDispatcher;

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskDispatcher<C> for DefaultTaskDispatcher {
    async fn dispatch(
        &self,
        handle: &TaskHandle<C>,
    ) -> Result<(), C::TaskError> {
        tokio::select! {
            result = handle.run() => result,
            _ = handle.cancelled() => Ok(()),
        }
    }
}
