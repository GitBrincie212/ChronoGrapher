pub mod ephemeral;
// skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::ErasedTask;
pub use ephemeral::*;
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type Key: Into<usize> + Debug + Hash + Eq + PartialEq + Clone + Send + Sync;

    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn get(&self, key: &Self::Key) -> Option<Arc<ErasedTask<C::TaskError>>>;

    fn exists(&self, key: &Self::Key) -> bool;

    fn store(
        &self,
        task: Arc<ErasedTask<C::TaskError>>,
    ) -> Result<Self::Key, Box<dyn Error + Send + Sync>>;

    fn remove(&self, key: &Self::Key);

    fn clear(&self);
}