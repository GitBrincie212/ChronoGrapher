pub mod default; // skipcq: RS-D1001

use crate::scheduler::{SchedulerConfig, SchedulerKey};
use crate::task::ErasedTask;
pub use default::*;
use std::ops::Deref;

pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn dispatch(
        &self,
        id: &SchedulerKey<C>,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
    ) -> impl Future<Output = Result<(), C::TaskError>> + Send;

    fn cancel(&self, id: &SchedulerKey<C>) -> impl Future<Output = ()> + Send;
}