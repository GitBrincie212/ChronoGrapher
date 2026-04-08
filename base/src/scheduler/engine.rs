pub mod default;

pub use default::DefaultSchedulerEngine;

use crate::scheduler::{SchedulerConfig, SchedulerKey};
use std::error::Error;
use std::time::SystemTime;

pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn retrieve(&self) -> impl Future<Output = Vec<SchedulerKey<C>>> + Send;
    
    fn is_empty(&self) -> bool;

    fn clock(&self) -> &C::SchedulerClock;

    fn schedule(
        &self,
        id: &SchedulerKey<C>,
        time: SystemTime,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;
    
    fn clear(&self) -> impl Future<Output = ()> + Send;
}
