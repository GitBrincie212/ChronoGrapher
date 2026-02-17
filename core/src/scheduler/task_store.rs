pub mod ephemeral;
// skipcq: RS-D1001

pub use ephemeral::*;
use std::error::Error;
use std::ops::Deref;

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::{DynArcError, ErasedTask};
use async_trait::async_trait;
use std::time::SystemTime;

#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type StoredTask: Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static;

    async fn init(&self) {}

    async fn retrieve(&self) -> Option<(Self::StoredTask, SystemTime, C::TaskIdentifier)>;

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask>;

    async fn pop(&self);

    async fn exists(&self, idx: &C::TaskIdentifier) -> bool;

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask<C::TaskError>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>>;

    async fn remove(&self, idx: &C::TaskIdentifier);

    async fn clear(&self);
}
