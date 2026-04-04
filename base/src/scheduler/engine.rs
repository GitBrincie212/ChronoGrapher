pub mod default;

pub use default::DefaultSchedulerEngine;

use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

#[async_trait]
pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        async move {}
    }

    fn retrieve(&self) -> impl Future<Output = Vec<C::TaskIdentifier>> + Send;
    
    fn is_empty(&self) -> bool;

    fn clock(&self) -> &C::SchedulerClock;

    fn schedule(
        &self,
        id: &C::TaskIdentifier,
        time: SystemTime,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;
    
    fn clear(&self) -> impl Future<Output = ()> + Send;
}
