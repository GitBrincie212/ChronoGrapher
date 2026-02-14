pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001

use std::error::Error;
use std::marker::PhantomData;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{Task, TaskFrame, TaskTrigger};
use crate::utils::TaskIdentifier;
use std::sync::Arc;
use tokio::join;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub trait SchedulerConfig: Sized + 'static {
    type TaskIdentifier: TaskIdentifier;
    type Error: Error + Send + Sync + 'static;
    type SchedulerClock: SchedulerClock<Self>;
    type SchedulerTaskStore: SchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher: SchedulerTaskDispatcher<Self>;
    type SchedulerEngine: SchedulerEngine<Self>;
}

pub struct DefaultSchedulerConfig<E: Error + Send + Sync + 'static>(PhantomData<E>);

impl<E: Error + Send + Sync + 'static> SchedulerConfig for DefaultSchedulerConfig<E> {
    type TaskIdentifier = Uuid;
    type Error = E;
    type SchedulerClock = ProgressiveClock;
    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher;
    type SchedulerEngine = DefaultSchedulerEngine;
}

#[allow(unused_variables)]
#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<T>))]
pub struct SchedulerInitConfig<T: SchedulerConfig> {
    dispatcher: T::SchedulerTaskDispatcher,

    store: T::SchedulerTaskStore,

    clock: T::SchedulerClock,

    engine: T::SchedulerEngine,
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for Scheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        Self {
            dispatcher: Arc::new(config.dispatcher),
            store: Arc::new(config.store),
            clock: Arc::new(config.clock),
            process: Mutex::new(None),
            engine: Arc::new(config.engine),
        }
    }
}

pub enum SchedulerHandleInstructions<C: SchedulerConfig> {
    Reschedule(C::TaskIdentifier), // Forces the Task to reschedule (instances may still run)
    Halt(C::TaskIdentifier), // Cancels the Task's current execution, if any
    Block(C::TaskIdentifier), // Blocks the Task from rescheduling
}

pub struct Scheduler<C: SchedulerConfig> {
    clock: Arc<C::SchedulerClock>,
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: Mutex<Option<JoinHandle<()>>>,
}

impl<C, E> Default for Scheduler<C>
where
    C: SchedulerConfig<
        SchedulerTaskStore: Default,
        SchedulerTaskDispatcher: Default,
        SchedulerEngine: Default,
        SchedulerClock: Default,
        Error = E
    >,
    E: Error + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::builder()
            .store(C::SchedulerTaskStore::default())
            .clock(C::SchedulerClock::default())
            .engine(C::SchedulerEngine::default())
            .dispatcher(C::SchedulerTaskDispatcher::default())
            .build()
    }
}

impl<C: SchedulerConfig> Scheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }

    pub async fn start(&self) {
        let process_lock = self.process.lock().await;
        if process_lock.is_some() {
            return;
        }
        drop(process_lock);
        join!(
            self.clock.init(),
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init()
        );
        let engine_clone = self.engine.clone();
        let clock_clone = self.clock.clone();
        let store_clone = self.store.clone();
        let dispatcher_clone = self.dispatcher.clone();
        *self.process.lock().await = Some(tokio::spawn(async move {
            engine_clone
                .main(clock_clone, store_clone, dispatcher_clone)
                .await;
        }))
    }

    pub async fn abort(&self) {
        let process = self.process.lock().await.take();
        if let Some(p) = process {
            p.abort();
        }
    }

    pub async fn clear(&self) {
        self.store.clear().await;
    }

    pub async fn schedule(
        &self,
        task: &Task<impl TaskFrame<Error = C::Error>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, impl Error + Send + Sync> {
        let erased = task.as_erased();
        self.store.store(&self.clock, erased).await
    }

    pub async fn cancel(&self, idx: &C::TaskIdentifier) {
        self.store.remove(idx).await;
    }

    pub async fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.store.exists(idx).await
    }

    pub async fn has_started(&self) -> bool {
        self.process.lock().await.is_some()
    }
}
