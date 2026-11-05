use crate::errors::TaskError;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::{BoundedTaskHandle, SchedulerTaskStore};
use crate::storage::{TaskHandle, TaskHookHandle};
use crate::task::{
    BoxedDynTaskFrame, BoxedDynTaskSchedule, DynTaskFrame, TaskFrame, TaskHook, TaskHookEvent,
    TaskSchedule,
};
use crossbeam::utils::CachePadded;
use parking_lot::MappedRwLockReadGuard;
use slotmap::{SecondaryMap, SlotMap, new_key_type};
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Weak};

type CachePaddedLock<T> = CachePadded<parking_lot::RwLock<T>>;
type TaskFrameStore<E> = [SlotMap<TaskKey, BoxedDynTaskFrame<E>>];
type TaskScheduleStore = [SecondaryMap<TaskKey, BoxedDynTaskSchedule>];

new_key_type! {struct TaskKey;}

pub type ErasedEphemeralHandle<T> =
    EphemeralTaskHandle<BoxedDynTaskFrame<<T as DynTaskFrame>::Error>, BoxedDynTaskSchedule>;

pub struct EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
    key: TaskKey,
    by_frame: Weak<TaskFrameStore<T1::Error>>,
    by_schedule: Weak<TaskScheduleStore>,
    _marker: PhantomData<fn() -> (T1, T2)>,
}

impl<T1, T2> Debug for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_tuple("EphemeralTaskHandle");
        debug_struct.field(&self.key);
        debug_struct.finish()
    }
}

impl<T1, T2> Clone for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            by_frame: self.by_frame.clone(),
            by_schedule: self.by_schedule.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T1, T2> PartialEq for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && Weak::ptr_eq(&self.by_frame, &other.by_frame)
    }
}

impl<T1, T2> Eq for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
}

impl<T1, T2> Hash for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.type_id().hash(state);
    }
}

macro_rules! create_guard {
    ($name: ident($store: ty) where T: $($bounds: tt)+) => {
        pub struct $name<T: $($bounds)+> {
            guard: MappedRwLockReadGuard<'static, T>,
            store: Arc<$store>
        }

        impl<T: $($bounds)+> $name<T> {
            fn new(
                store: Arc<$store>,
                guard: MappedRwLockReadGuard<'_, T>,
            ) -> Self {
                // SAFETY: "guard" borrows from the lock contained inside "store".
                // "store" is moved into this struct and therefore keeps the
                // underlying allocation alive for as long as "guard" exists.
                // "guard" is declared before "store", so "guard" is dropped
                // before `store`.
                //
                // Therefore, extending the guard lifetime to 'static is safe.
                let guard = unsafe {
                    std::mem::transmute::<
                        MappedRwLockReadGuard<'_, T>,
                        MappedRwLockReadGuard<'static, T>,
                    >(guard)
                };

                Self { guard, store }
            }
        }

        impl<T: $($bounds)+> Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &T {
                &self.guard
            }
        }
    };
}

create_guard!(TaskScheduleGuard(TaskScheduleStore) where T: Any + Send + Sync);
create_guard!(TaskFrameGuard(TaskFrameStore<T::Error>) where T: DynTaskFrame);

pub struct EphemeralHookHandle<T: Any + Send + Sync> {
    _marker: PhantomData<T>,
}

impl<T: Any + Send + Sync> Hash for EphemeralHookHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<T>().hash(state)
    }
}

impl<T: Any + Send + Sync> Clone for EphemeralHookHandle<T> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Any + Send + Sync> PartialEq for EphemeralHookHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<T: Any + Send + Sync> Eq for EphemeralHookHandle<T> {}

impl<T: Any + Send + Sync> TaskHookHandle<T> for EphemeralHookHandle<T> {
    type TaskHookView = ();

    fn get(&self) -> Option<Self::TaskHookView> {
        todo!()
    }

    fn subscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>,
    {
        todo!()
    }

    fn unsubscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>,
    {
        todo!()
    }

    fn emit<EV: TaskHookEvent>(payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send
    where
        T: TaskHook<EV>,
    {
        todo!()
    }

    fn detach<EV: TaskHookEvent>() -> impl Future<Output = ()> + Send {
        todo!()
    }
}

impl<T1, T2> TaskHandle for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Any + Send + Sync,
{
    type TaskFrame = T1;
    type TaskSchedule = T2;
    type TaskHookHandle<T: Any + Send + Sync> = EphemeralHookHandle<T>;
    type TaskFrameView = TaskFrameGuard<Self::TaskFrame>;
    type TaskScheduleView = TaskScheduleGuard<Self::TaskSchedule>;
    type ErasedTaskHandle = EphemeralTaskHandle<BoxedDynTaskFrame<T1::Error>, BoxedDynTaskSchedule>;

    fn get_frame(&self) -> Option<Self::TaskFrameView> {
        todo!()
    }

    fn get_schedule(&self) -> Option<Self::TaskScheduleView> {
        todo!()
    }

    fn attach_hook<TH: Any + Send + Sync>(
        &self,
        hook: TH,
    ) -> impl Future<Output = Self::TaskHookHandle<TH>> {
        todo!()
    }

    fn detach_hook<TH: Any + Send + Sync>(&self) -> impl Future<Output = ()> + Send {
        todo!()
    }

    fn emit<EV: TaskHookEvent>(
        &self,
        payload: &EV::Payload<'_>,
    ) -> impl Future<Output = ()> + Send {
        todo!()
    }

    fn get_hook<TH: TaskHook<EV>, EV: TaskHookEvent>(&self) -> Option<Self::TaskHookHandle<TH>> {
        todo!()
    }

    fn erase(&self) -> Self::ErasedTaskHandle
    where
        Self::TaskFrame: TaskFrame,
        Self::TaskSchedule: TaskSchedule,
    {
        todo!()
    }

    fn is_invalid(&self) -> bool {
        todo!()
    }

    fn invalidate(&self) -> impl Future<Output = ()> + Send {
        todo!()
    }
}

impl<T1, T2> BoundedTaskHandle for EphemeralTaskHandle<T1, T2>
where
    T1: DynTaskFrame<Args = ()>,
    T2: Any + Send + Sync,
{
    fn schedule(&self) -> impl Future<Output = ()> {
        todo!()
    }

    fn cancel(&self) -> impl Future<Output = ()> + Send {
        todo!()
    }
}

pub struct EphemeralTaskStoreShard<C: SchedulerConfig> {}

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    by_frame: Arc<TaskFrameStore<C::TaskError>>,
    by_schedule: Arc<TaskScheduleStore>,
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism().unwrap().get();

        let shard_count = (parallelism * 4).next_power_of_two();
        todo!()
    }
}

impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type TaskHandle<T1, T2>
        = EphemeralTaskHandle<T1, T2>
    where
        T1: DynTaskFrame<Args = (), Error = C::TaskError>,
        T2: Send + Sync + 'static;

    fn allocate<T1, T2>(
        &self,
        frame: T1,
        schedule: T2,
    ) -> Result<Self::TaskHandle<T1, T2>, Box<dyn TaskError>>
    where
        T1: TaskFrame<Args = (), Error = C::TaskError>,
        T2: TaskSchedule,
    {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }
}
