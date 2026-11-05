pub mod ephemeral; // skipcq: RS-D1001

use crate::errors::TaskError;
use crate::scheduler::SchedulerConfig;
use crate::storage::TaskHandle;
use crate::task::{
    BoxedDynTaskFrame, BoxedDynTaskSchedule, DynTaskFrame, OnTaskEnd, OnTaskStart,
    RestrictTaskFrameContext, TaskFrame, TaskFrameContext, TaskSchedule,
};
pub use ephemeral::*;
use std::any::Any;

pub type TaskHandleAlias<C, T1, T2> =
    <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskHandle<T1, T2>;
pub type ErasedTaskHandleAlias<C> =
    TaskHandleAlias<C, BoxedDynTaskFrame<<C as SchedulerConfig>::TaskError>, BoxedDynTaskSchedule>;

trait Sealed {}
pub trait TaskHandleExt: Sealed {
    type Error: TaskError;

    fn run(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait BoundedTaskHandle: TaskHandle {
    fn schedule(&self) -> impl Future<Output = ()> + Send;
    fn cancel(&self) -> impl Future<Output = ()> + Send;
}

impl<T: BoundedTaskHandle> Sealed for T {}
impl<T: BoundedTaskHandle> TaskHandleExt for T {
    type Error = <T::TaskFrame as DynTaskFrame>::Error;

    async fn run(&self) -> Result<(), Self::Error> {
        let Some(frame) = self.get_frame() else {
            return Ok(());
        };

        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015

        let result = frame.dyn_execute(&ctx, &()).await;
        let err = match &result {
            Ok(_) => None,
            Err(e) => Some(e as &dyn TaskError),
        };

        ctx.emit::<OnTaskEnd>(&err).await;
        result
    }
}

pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type TaskHandle<T1, T2>: BoundedTaskHandle<
            TaskFrame = T1,
            TaskSchedule = T2,
            ErasedTaskHandle = Self::TaskHandle<
                BoxedDynTaskFrame<C::TaskError>,
                BoxedDynTaskSchedule,
            >,
        >
    where
        T1: DynTaskFrame<Args = (), Error = C::TaskError>,
        T2: Any + Send + Sync;

    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn allocate<T1, T2>(
        &self,
        frame: T1,
        schedule: T2,
    ) -> Result<Self::TaskHandle<T1, T2>, Box<dyn TaskError>>
    where
        T1: TaskFrame<Args = (), Error = C::TaskError>,
        T2: TaskSchedule;

    fn clear(&self);
}
