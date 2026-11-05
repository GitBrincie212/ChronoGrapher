use crate::errors::TaskError;
use crate::task::{
    BoxedDynTaskFrame, BoxedDynTaskSchedule, DynTaskFrame, TaskFrame, TaskHook, TaskHookEvent,
    TaskSchedule,
};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

pub trait TaskHookHandle<T: Any + Send + Sync>:
    Hash + Clone + PartialEq + Eq + Any + Send + Sync
{
    type TaskHookView: Deref<Target = T>;

    fn get(&self) -> Option<Self::TaskHookView>;

    fn subscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>;

    fn unsubscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>;

    fn emit<EV: TaskHookEvent>(payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>;

    fn detach<EV: TaskHookEvent>() -> impl Future<Output = ()> + Send;
}

pub trait TaskHandle: Debug + Hash + Clone + PartialEq + Eq + Any + Send + Sync {
    type TaskFrame: DynTaskFrame<Args = ()>;
    type TaskSchedule: Any + Send + Sync;
    type TaskHookHandle<T: Any + Send + Sync>: TaskHookHandle<T>;
    type TaskFrameView: Deref<Target = Self::TaskFrame> + Send;
    type TaskScheduleView: Deref<Target = Self::TaskSchedule> + Send;
    type ErasedTaskHandle: TaskHandle<
            TaskFrame = BoxedDynTaskFrame<<Self::TaskFrame as DynTaskFrame>::Error>,
            TaskSchedule = BoxedDynTaskSchedule,
        >;

    fn get_frame(&self) -> Option<Self::TaskFrameView>;
    fn get_schedule(&self) -> Option<Self::TaskScheduleView>;

    fn attach_hook<TH: Any + Send + Sync>(
        &self,
        hook: TH,
    ) -> impl Future<Output = Self::TaskHookHandle<TH>> + Send;
    fn detach_hook<TH: Any + Send + Sync>(&self) -> impl Future<Output = ()> + Send;
    fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>)
    -> impl Future<Output = ()> + Send;
    fn get_hook<TH: TaskHook<EV>, EV: TaskHookEvent>(&self) -> Option<Self::TaskHookHandle<TH>>;

    fn erase(&self) -> Self::ErasedTaskHandle
    where
        Self::TaskFrame: TaskFrame,
        Self::TaskSchedule: TaskSchedule;

    fn is_invalid(&self) -> bool;
    fn invalidate(&self) -> impl Future<Output = ()> + Send;
}

pub trait TaskStorage: Any + Send + Sync {
    type TaskHandle<T1, T2>;

    fn allocate<T1, T2>(
        &self,
        frame: T1,
        schedule: T2,
    ) -> Result<Self::TaskHandle<T1, T2>, Box<dyn TaskError>>
    where
        T1: TaskFrame<Args = ()>,
        T2: TaskSchedule;
}

pub trait MigratableFrom<S: TaskStorage>: TaskStorage {
    fn migrate<T1, T2>(
        &self,
        handle: S::TaskHandle<T1, T2>,
    ) -> impl Future<Output = Self::TaskHandle<T1, T2>>
    where
        T1: TaskFrame<Args = ()>,
        T2: TaskSchedule;
}
