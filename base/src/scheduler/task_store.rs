pub mod ephemeral;
// skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
pub use ephemeral::*;
use std::ops::Deref;
use std::hash::Hash;
use crate::errors::TaskError;
use crate::task::{DynTaskFrame, StaleTaskHook, TaskDefinitions, TaskFrame, TaskHook, TaskHookEvent, TaskTrigger};

pub type SchedulerKey<C> = <SchedulerTaskRef<C> as TaskRef>::Key;
pub type SchedulerTaskRef<C> = <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskRef;
pub type SchedulerTaskHookHandle<C, T> = <SchedulerTaskRef<C> as TaskRef>::TaskHookHandle<T>;

pub trait TaskHookHandle: Eq + PartialEq + Clone + Send + Sync {
    fn get<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output = &T> + Send;
    fn get_dyn(&self) -> impl Future<Output = &dyn TaskHook<()>> + Send;

    fn subscribe<E: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send;
    fn unsubscribe<E: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send;
    fn emit<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) -> impl Future<Output = ()> + Send;

    fn is_valid(&self) -> impl Future<Output = bool> + Send;
    fn detach(&self) -> impl Future<Output = ()> + Send;
}

pub trait TaskRef: Eq + PartialEq + Clone + Send + Sync {
    type TaskError: TaskError;
    type Key: Hash + PartialEq + Eq + Clone + Send + Sync;
    type TaskTrigger: Deref<Target = dyn TaskTrigger>;
    type TaskFrame: Deref<Target = dyn DynTaskFrame<Self::TaskError>>;
    type TaskHookHandle<T: StaleTaskHook>: TaskHookHandle;

    fn frame(&self) -> impl Future<Output = Option<&Self::TaskFrame>> + Send;
    fn trigger(&self) -> impl Future<Output = Option<&Self::TaskTrigger>> + Send;

    fn attach_hook<T: StaleTaskHook>(&self, value: T) -> impl Future<Output = Option<Self::TaskHookHandle<T>>> + Send;
    fn detach_hook<T: StaleTaskHook>(&self) -> impl Future<Output = ()> + Send;
    fn detach_hook_from<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output = ()> + Send;
    fn get_hook_from<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output = Option<Self::TaskHookHandle<T>>> + Send;
    fn get_hook<T: StaleTaskHook>(&self) -> impl Future<Output = Option<Self::TaskHookHandle<T>>> + Send;
    fn emit_event<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) -> impl Future<Output = ()> + Send;

    fn is_valid(&self) -> impl Future<Output = bool> + Send;
    fn invalidate(&self) -> impl Future<Output = ()> + Send;
    fn key(&self) -> &Self::Key;
}

pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    type TaskRef: TaskRef<TaskError = C::TaskError>;

    fn init(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn allocate<T1: TaskFrame<Error = C::TaskError>, T2: TaskTrigger>(
        &self,
        task: TaskDefinitions<T1, T2>,
    ) -> impl Future<Output = Self::TaskRef> + Send;

    fn resolve(
        &self, key: &<Self::TaskRef as TaskRef>::Key
    ) -> impl Future<Output = Option<Self::TaskRef>> + Send;

    async fn clear(&self);
}