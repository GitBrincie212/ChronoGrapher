pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001

use crate::errors::TaskError;
use crate::prelude::TaskHook;
use crate::scheduler::clock::*;
use crate::scheduler::engine::default::SchedulerHandleInstructions;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine, SchedulerHandlePayload};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ErasedTask, Task, TaskFrame, TaskTrigger};
use crate::utils::TaskIdentifier;
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::join;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub type DefaultScheduler<E> = Scheduler<DefaultSchedulerConfig<E>>;

#[cfg(feature = "anyhow")]
pub type DefaultAnyhowScheduler = DefaultScheduler<anyhow::Error>;

#[cfg(feature = "eyre")]
pub type DefaultEyreScheduler = DefaultScheduler<eyre::Error>;

pub trait SchedulerConfig: Sized + 'static {
    type TaskIdentifier: TaskIdentifier;
    type TaskError: TaskError;
    type SchedulerClock: SchedulerClock<Self>;
    type SchedulerTaskStore: SchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher: SchedulerTaskDispatcher<Self>;
    type SchedulerEngine: SchedulerEngine<Self>;
}

pub struct DefaultSchedulerConfig<E: TaskError>(PhantomData<E>);

impl<E: TaskError> SchedulerConfig for DefaultSchedulerConfig<E> {
    type TaskIdentifier = Uuid;
    type TaskError = E;
    type SchedulerClock = ProgressiveClock;
    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher;
    type SchedulerEngine = DefaultSchedulerEngine;
}

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
            instruction_channel: Mutex::new(None),
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    clock: Arc<C::SchedulerClock>,
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: Mutex<Option<JoinHandle<()>>>,
    instruction_channel: Mutex<Option<tokio::sync::mpsc::Sender<SchedulerHandlePayload>>>,
}

impl<C> Default for Scheduler<C>
where
    C: SchedulerConfig<
            SchedulerTaskStore: Default,
            SchedulerTaskDispatcher: Default,
            SchedulerEngine: Default,
            SchedulerClock: Default,
            TaskError: TaskError,
        >,
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

pub(crate) struct SchedulerHandle {
    pub(crate) id: Arc<dyn Any + Send + Sync>,
    pub(crate) channel: tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
}

impl SchedulerHandle {
    pub(crate) async fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel
            .send((self.id.clone(), instruction))
            .await
            .expect("Cannot instruct");
    }
}

impl TaskHook<()> for SchedulerHandle {}

pub(crate) async fn append_scheduler_handler<C: SchedulerConfig>(
    task: &ErasedTask<C::TaskError>,
    id: C::TaskIdentifier,
    channel: &tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
) {
    let handle = SchedulerHandle {
        id: Arc::new(id),
        channel: channel.clone(),
    };

    task.attach_hook::<()>(Arc::new(handle)).await;
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

        let engine_clone = self.engine.clone();
        let clock_clone = self.clock.clone();
        let store_clone = self.store.clone();
        let dispatcher_clone = self.dispatcher.clone();

        let channel = join!(
            self.clock.init(),
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init(),
            self.engine
                .create_instruction_channel(&clock_clone, &store_clone, &dispatcher_clone)
        )
        .4;

        for (id, task) in self.store.iter() {
            append_scheduler_handler::<C>(&task, id, &channel).await;
        }

        *self.instruction_channel.lock().await = Some(channel);

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
        task: &Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let erased = task.as_erased();
        let id = C::TaskIdentifier::generate();
        if let Some(channel) = &*self.instruction_channel.lock().await {
            append_scheduler_handler::<C>(&erased, id.clone(), channel).await;
        }
        self.store.store(&self.clock, id, erased).await
    }

    pub async fn cancel(&self, idx: &C::TaskIdentifier) {
        self.store.remove(idx).await;
    }

    pub fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.store.exists(idx)
    }

    pub async fn has_started(&self) -> bool {
        self.process.lock().await.is_some()
    }
}
