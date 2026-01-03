pub mod default;
pub use default::DefaultSchedulerEngine;

use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use std::sync::Arc;

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
