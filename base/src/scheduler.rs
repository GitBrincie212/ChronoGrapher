pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001
pub mod impls; // skipcq: RS-D1001

pub use impls::*;

use crate::errors::TaskError;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::{EphemeralSchedulerTaskStore,SchedulerTaskStore };
use crate::scheduler::utils::SchedulerHandleInstructions;
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use crate::task::{Task, TaskFrame, TaskTrigger};

pub type SchedulerKey<C> = <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::Key;

pub(crate) type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);

#[cfg(feature = "anyhow")]
pub type DefaultAnyhowScheduler = DefaultScheduler<anyhow::Error>;

#[cfg(feature = "eyre")]
pub type DefaultEyreScheduler = DefaultScheduler<eyre::Error>;

pub trait SchedulerConfig: Sized + 'static {
    type TaskError: TaskError;

    type SchedulerTaskStore: SchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher: SchedulerTaskDispatcher<Self>;
    type SchedulerEngine: SchedulerEngine<Self>;
    type SchedulerClock: SchedulerClock;
}

pub struct DefaultSchedulerConfig<E: TaskError>(PhantomData<E>);

impl<E: TaskError> SchedulerConfig for DefaultSchedulerConfig<E> {
    type TaskError = E;

    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher<Self>;
    type SchedulerEngine = DefaultSchedulerEngine<Self>;
    type SchedulerClock = ProgressiveClock;
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FailoverPolicy {
    Keep,

    #[default]
    Terminate,
    Deallocate,
    ShutdownScheduler
}

pub trait Scheduler<C: SchedulerConfig>: Sync + Send + 'static {
    type Handle: Into<SchedulerKey<C>>;

    fn start(&self) -> impl Future<Output = ()> + Send;
    fn has_started(&self) -> impl Future<Output = bool> + Send;
    fn abort(&self) -> impl Future<Output = ()> + Send;

    fn exists(&self, key: &Self::Handle) -> impl Future<Output = bool> + Send;

    fn schedule<T1, T2>(
        &self,
        task: Task<T1, T2>,
    ) -> impl Future<Output = Result<Self::Handle, Box<dyn Error + Send + Sync>>>
    where
        T1: TaskFrame<Args = (), Error = C::TaskError>,
        T2: TaskTrigger;

    fn remove(&self, key: &Self::Handle) -> impl Future<Output = ()> + Send;

    fn clear(&self) -> impl Future<Output = ()> + Send;
}