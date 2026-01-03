pub mod default;
pub use default::DefaultSchedulerEngine;

use std::sync::Arc;
use async_trait::async_trait;
use crate::scheduler::SchedulerConfig;

#[async_trait]
pub trait SchedulerEngine<F: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}
    async fn main(
        &self,
        clock: Arc<F::SchedulerClock>,
        store: Arc<F::SchedulerTaskStore>,
        dispatcher: Arc<F::SchedulerTaskDispatcher>,
    );
}