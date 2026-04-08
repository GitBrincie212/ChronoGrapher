pub mod default; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
use crate::task::ErasedTask;
use async_trait::async_trait;
pub use default::*;
use std::ops::Deref;
use crate::scheduler::task_store::SchedulerTaskRef;

#[async_trait]
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn dispatch(
        &self,
        id: &SchedulerTaskRef<C>,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
    ) -> Result<(), C::TaskError>;

    async fn cancel(&self, id: &SchedulerTaskRef<C>);
}