pub mod ephemeral;
// skipcq: RS-D1001

use std::any::Any;
pub use ephemeral::*;
use std::error::Error;
use std::ops::Deref;

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::time::SystemTime;

pub type SchedulePayload = (Box<dyn Any + Send + Sync>, Result<SystemTime, Box<dyn Error + Send + Sync>>);

pub enum RescheduleError {
    Success,
    TriggerError(Box<dyn Error + Send + Sync>),
    UnknownTask
}

#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type StoredTask: Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static;

    async fn init(&self) {}

    async fn retrieve(&self) -> (Self::StoredTask, SystemTime, C::TaskIdentifier);

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask>;

    async fn pop(&self);

    async fn exists(&self, idx: &C::TaskIdentifier) -> bool;

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> RescheduleError;

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        id: C::TaskIdentifier,
        task: ErasedTask<C::TaskError>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>>;

    async fn remove(&self, idx: &C::TaskIdentifier);

    async fn clear(&self);
}
