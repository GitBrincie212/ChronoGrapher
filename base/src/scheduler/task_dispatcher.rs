pub mod default; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::ErasedTaskHandleAlias;
pub use default::*;

pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn dispatch(
        &self,
        handle: &ErasedTaskHandleAlias<C>,
    ) -> impl Future<Output = Result<(), C::TaskError>> + Send;

    fn cancel(&self, id: &ErasedTaskHandleAlias<C>) -> impl Future<Output = ()> + Send;
}
