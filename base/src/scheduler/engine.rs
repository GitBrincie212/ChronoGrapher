pub mod default;

pub use default::DefaultSchedulerEngine;

use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::ErasedTaskHandleAlias;
use std::error::Error;
use std::time::SystemTime;

pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn retrieve(&self) -> impl Future<Output = Vec<ErasedTaskHandleAlias<C>>> + Send;

    fn clock(&self) -> &C::SchedulerClock;

    fn schedule(
        &self,
        id: &ErasedTaskHandleAlias<C>,
        time: SystemTime,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send;

    fn clear(&self) -> impl Future<Output = ()> + Send;
}
