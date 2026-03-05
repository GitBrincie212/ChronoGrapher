pub mod ephemeral;
// skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::ErasedTask;
use async_trait::async_trait;
pub use ephemeral::*;
use std::error::Error;
use std::ops::Deref;

#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type StoredTask: Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static;

    async fn init(&self) {}
    
    fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask>;

    fn exists(&self, idx: &C::TaskIdentifier) -> bool;

    fn store(
        &self,
        id: &C::TaskIdentifier,
        task: ErasedTask<C::TaskError>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    fn remove(&self, idx: &C::TaskIdentifier);

    fn clear(&self);

    fn iter(&self) -> impl Iterator<Item = (C::TaskIdentifier, Self::StoredTask)>;
}
