pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod impls;
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001 // skipcq: RS-D1001

pub use impls::*;

use crate::errors::TaskError;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::{EphemeralSchedulerTaskStore, SchedulerTaskStore};
use crate::scheduler::utils::SchedulerHandleInstructions;
use crate::task::{TaskFrame, TaskSchedule};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

pub(crate) type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);

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

#[cfg(feature = "anyhow")]
pub trait AnyhowSchedulerConfig {}

#[cfg(feature = "anyhow")]
impl<T: SchedulerConfig<TaskError = anyhow::Error>> AnyhowSchedulerConfig for T {}

#[cfg(feature = "anyhow")]
pub trait EyreSchedulerConfig {}

#[cfg(feature = "anyhow")]
impl<T: SchedulerConfig<TaskError = eyre::Error>> EyreSchedulerConfig for T {}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FailoverPolicy {
    Keep,

    #[default]
    Terminate,
    Deallocate,
    ShutdownScheduler,
}

pub type AllocateResult<C, T1, T2> = Result<
    <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskHandle<T1, T2>,
    Box<dyn TaskError>,
>;

pub trait Scheduler<C: SchedulerConfig>: Sync + Send + 'static {
    fn start(&self) -> impl Future<Output = ()> + Send;
    fn has_started(&self) -> impl Future<Output = bool> + Send;
    fn abort(&self) -> impl Future<Output = ()> + Send;

    fn allocate<T1, T2>(
        &self,
        frame: T1,
        schedule: T2,
    ) -> impl Future<Output = AllocateResult<C, T1, T2>>
    where
        T1: TaskFrame<Args = (), Error = C::TaskError>,
        T2: TaskSchedule;

    fn clear(&self) -> impl Future<Output = ()> + Send;
}
