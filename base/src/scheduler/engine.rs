pub mod default;

pub use default::DefaultSchedulerEngine;

use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

#[async_trait]
pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    async fn retrieve(&self) -> Vec<C::TaskIdentifier>;
    
    fn is_empty(&self) -> bool;

    async fn init(&self) {}

    fn clock(&self) -> &C::SchedulerClock;

    async fn schedule(
        &self,
        id: &C::TaskIdentifier,
        time: SystemTime,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    async fn clear(&self);
}
