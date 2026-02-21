pub mod default;

pub use default::DefaultSchedulerEngine;
use std::any::Any;

use crate::scheduler::SchedulerConfig;
use crate::scheduler::engine::default::SchedulerHandleInstructions;
use async_trait::async_trait;
use std::sync::Arc;

pub type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);

#[async_trait]
pub trait SchedulerEngine<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}
    async fn main(
        &self,
        clock: Arc<C::SchedulerClock>,
        store: Arc<C::SchedulerTaskStore>,
        dispatcher: Arc<C::SchedulerTaskDispatcher>,
    );
    async fn create_instruction_channel(
        &self,
        clock: &Arc<C::SchedulerClock>,
        store: &Arc<C::SchedulerTaskStore>,
        dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    ) -> tokio::sync::mpsc::Sender<SchedulerHandlePayload>;
}
