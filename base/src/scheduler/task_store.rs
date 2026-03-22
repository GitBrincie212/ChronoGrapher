pub mod ephemeral;
// skipcq: RS-D1001

use std::error::Error;
use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::ErasedTask;
use async_trait::async_trait;
pub use ephemeral::*;
use crate::task::{TaskFrame, TaskRef, TaskTrigger};

#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type TaskRef: TaskRef<C>;

    async fn init(&self) {}

    async fn allocate(
        &self,
        trigger: impl TaskTrigger,
        frame: impl TaskFrame<Error = C::TaskError>
    ) -> Result<Self::TaskRef, Box<dyn Error + Send + Sync>>;

    async fn deallocate(&self, handle: &Self::TaskRef);

    async fn clear(&self);
}
