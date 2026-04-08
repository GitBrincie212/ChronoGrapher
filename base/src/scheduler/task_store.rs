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

pub trait TaskHookHandle<T: StaleTaskHook>: Eq + PartialEq + Clone + Send + Sync {
    fn get(&self) -> impl Future<Output = Option<&T>> + Send;
    fn get_from<EV: TaskHookEvent>(&self) -> impl Future<Output = Option<&T>> + Send;

    fn subscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send;
    fn unsubscribe<EV: TaskHookEvent>(&self) -> impl Future<Output = ()> + Send;
    fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send;

    fn is_valid(&self) -> impl Future<Output = bool> + Send;
    fn detach(&self) -> impl Future<Output = ()> + Send;
}

pub trait TaskRef: Eq + PartialEq + Clone + Send + Sync {
    type TaskError: TaskError;
    type Key: Hash + PartialEq + Eq + Clone + Send + Sync;
    type TaskTrigger: Deref<Target = dyn TaskTrigger>;
    type TaskFrame: Deref<Target = dyn DynTaskFrame<Self::TaskError>>;
    type TaskHookHandle<TH: StaleTaskHook>: TaskHookHandle<TH>;

    fn frame(&self) -> impl Future<Output = Option<&Self::TaskFrame>> + Send;
    fn trigger(&self) -> impl Future<Output = Option<&Self::TaskTrigger>> + Send;

    fn attach_hook<TH: StaleTaskHook>(&self, value: TH) -> impl Future<Output = Self::TaskHookHandle<TH>> + Send;
    fn detach_hook<TH: StaleTaskHook>(&self) -> impl Future<Output = ()> + Send;
    fn detach_hook_from<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send;
    fn get_hook_from<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = Self::TaskHookHandle<T>> + Send;
    fn get_hook<TH: StaleTaskHook>(&self) -> impl Future<Output = Self::TaskHookHandle<TH>> + Send;
    fn emit_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send;

    fn is_valid(&self) -> impl Future<Output = bool> + Send;
    fn deallocate(&self) -> impl Future<Output = ()> + Send;
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