pub mod default; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
use crate::task::ErasedTask;
pub use default::*;
use std::ops::Deref;

pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        async move {}
    }

    fn dispatch(
        &self,
        id: &C::TaskIdentifier,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
    ) -> impl Future<Output = Result<(), C::TaskError>> + Send;

    fn cancel(&self, id: &C::TaskIdentifier) -> impl Future<Output = ()> + Send;
}