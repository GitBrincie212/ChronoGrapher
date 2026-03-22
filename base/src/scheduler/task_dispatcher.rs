pub mod default; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
use crate::task::{ErasedTask, TaskHandle};
use async_trait::async_trait;
pub use default::*;
use std::ops::Deref;

#[async_trait]
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn dispatch(
        &self,
        id: &TaskHandle<C>,
    ) -> Result<(), C::TaskError>;
}
