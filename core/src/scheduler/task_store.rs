pub mod ephemeral;
// skipcq: RS-D1001

pub use ephemeral::*;

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::{ErasedTask, DynArcError};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;

#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn retrieve(&self) -> Option<(Arc<ErasedTask<C::Error>>, SystemTime, C::TaskIdentifier)>;

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Arc<ErasedTask<C::Error>>>;

    async fn pop(&self);

    async fn exists(&self, idx: &C::TaskIdentifier) -> bool;

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> Result<(), DynArcError>;

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask<C::Error>,
    ) -> Result<C::TaskIdentifier, DynArcError>;

    async fn remove(&self, idx: &C::TaskIdentifier);

    async fn clear(&self);
}
