pub mod default;

pub use default::DefaultSchedulerEngine;

use std::error::Error;
use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use std::time::SystemTime;

#[async_trait]
pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    async fn retrieve(&self) -> Vec<C::TaskIdentifier>;

    async fn init(&self) {}

    fn clock(&self) -> &C::SchedulerClock;

    async fn schedule(
        &self,
        id: &C::TaskIdentifier,
        time: SystemTime,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn cancel(&self, id: &C::TaskIdentifier);

    async fn clear(&self);
}
