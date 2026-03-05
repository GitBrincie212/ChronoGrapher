pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001
mod utils; // skipcq: RS-D1001

use crate::errors::TaskError;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, EngineNotifier, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{Task, TaskFrame, TaskTrigger};
use crate::utils::{DefaultTaskID, TaskIdentifier};
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::join;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

pub(crate) use crate::scheduler::utils::*;

pub const TRIGGER_WORKER_POOL: usize = 16;

pub(crate) type TriggerJobWorkers<C> = Vec<tokio::sync::mpsc::Sender<(
    <C as SchedulerConfig>::TaskIdentifier,
    Arc<dyn TaskTrigger>
)>>;
pub(crate) type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);
pub(crate) type ReschedulePayload<C> = (
    <C as SchedulerConfig>::TaskIdentifier,
    Option<<C as SchedulerConfig>::TaskError>
);

pub type DefaultScheduler<E> = Scheduler<DefaultSchedulerConfig<E>>;

#[cfg(feature = "anyhow")]
pub type DefaultAnyhowScheduler = DefaultScheduler<anyhow::Error>;

#[cfg(feature = "eyre")]
pub type DefaultEyreScheduler = DefaultScheduler<eyre::Error>;

pub trait SchedulerConfig: Sized + 'static {
    type TaskIdentifier: TaskIdentifier;
    type TaskError: TaskError;
    type SchedulerTaskStore: SchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher: SchedulerTaskDispatcher<Self>;
    type SchedulerEngine: SchedulerEngine<Self>;
    type SchedulerClock: SchedulerClock;
}

pub struct DefaultSchedulerConfig<E: TaskError>(PhantomData<E>);

impl<E: TaskError> SchedulerConfig for DefaultSchedulerConfig<E> {
    type TaskIdentifier = DefaultTaskID;
    type TaskError = E;
    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher<Self>;
    type SchedulerEngine = DefaultSchedulerEngine<Self>;
    type SchedulerClock = ProgressiveClock;
}

#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<T>))]
pub struct SchedulerInitConfig<T: SchedulerConfig> {
    dispatcher: T::SchedulerTaskDispatcher,
    store: T::SchedulerTaskStore,
    engine: T::SchedulerEngine,
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for Scheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        let engine = Arc::new(config.engine);
        let store = Arc::new(config.store);
        let mut workers = Vec::with_capacity(TRIGGER_WORKER_POOL);

        for _ in 0..TRIGGER_WORKER_POOL {
            let (tx, mut rx) =
                tokio::sync::mpsc::channel::<(C::TaskIdentifier, Arc<dyn TaskTrigger>)>(1024);
            let engine_clone = engine.clone();
            let store_clone = store.clone();
            tokio::spawn(async move {
                while let Some((id, trigger)) = rx.recv().await {
                    let now = engine_clone.clock().now();

                    match trigger.init(now).await {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("Initialization error from TaskTrigger: {:?}", err);
                            store_clone.remove(&id);
                            continue;
                        }
                    }

                    let time = match trigger.trigger(now).await {
                        Ok(time) => {
                            time
                        }
                        Err(err) => {
                            eprintln!("Computation error from TaskTrigger: {:?}", err);
                            store_clone.remove(&id);
                            continue;
                        }
                    };

                    match engine_clone.schedule(&id, time).await {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("Schedule error from SchedulerEngine: {:?}", err);
                            store_clone.remove(&id);
                            continue;
                        }
                    }
                }
            });

            workers.push(tx);
        }

        Self {
            engine,
            store,
            dispatcher: Arc::new(config.dispatcher),
            process: RwLock::new(None),
            instruction_channel: RwLock::new(None),
            trigger_workers: Arc::new(workers)
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: RwLock<Option<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>)>>,
    instruction_channel: RwLock<Option<tokio::sync::mpsc::Sender<SchedulerHandlePayload>>>,
    trigger_workers: Arc<TriggerJobWorkers<C>>
}

impl<C> Default for Scheduler<C>
where
    C: SchedulerConfig<
            SchedulerTaskStore: Default,
            SchedulerTaskDispatcher: Default,
            SchedulerEngine: Default,
            TaskError: TaskError,
        >,
{
    fn default() -> Self {
        Self::builder()
            .store(C::SchedulerTaskStore::default())
            .engine(C::SchedulerEngine::default())
            .dispatcher(C::SchedulerTaskDispatcher::default())
            .build()
    }
}

fn spawn_task<C: SchedulerConfig>(
    id: C::TaskIdentifier,
    scheduler_send: tokio::sync::mpsc::Sender<ReschedulePayload<C>>,
    dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    task: <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::StoredTask
) {
    let sender = EngineNotifier::new(
        id,
        scheduler_send,
    );

    let dispatcher = dispatcher.clone();
    tokio::spawn(async move {
        dispatcher.dispatch(task, &sender).await;
    });
}

impl<C: SchedulerConfig> Scheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }

    pub async fn start(&self) {
        let process_lock = self.process.read().await;
        if process_lock.is_some() {
            return;
        }
        drop(process_lock);

        let engine_clone = self.engine.clone();
        let store_clone = self.store.clone();
        let dispatcher_clone = self.dispatcher.clone();

        join!(
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init(&store_clone, &dispatcher_clone)
        );

        let (scheduler_send, scheduler_receive) =
            tokio::sync::mpsc::channel::<(C::TaskIdentifier, Option<C::TaskError>)>(20480);

        let (instruct_send, instruct_receive) =
            tokio::sync::mpsc::channel::<SchedulerHandlePayload>(1024);

        for (id, task) in self.store.iter() {
            append_scheduler_handler::<C>(&task, id, instruct_send.clone()).await;
        }

        *self.instruction_channel.write().await = Some(instruct_send);

        *self.process.write().await = Some((
            tokio::spawn(
                scheduler_handle_instructions_logic::<C>(
                    instruct_receive,
                    &dispatcher_clone,
                    &store_clone,
                    self.trigger_workers.clone(),
                    scheduler_send.clone()
                ),
            ),

            tokio::spawn(
                reschedule_logic::<C>(
                    &store_clone,
                    scheduler_receive,
                    self.trigger_workers.clone()
                )
            ),

            tokio::spawn(
                main_loop_logic::<C>(
                    &engine_clone,
                    &dispatcher_clone,
                    &store_clone,
                    scheduler_send,
                )
            )
        ));
    }

    pub async fn abort(&self) {
        let process = self.process.write().await.take();
        if let Some((p1, p2, p3)) = process {
            p1.abort();
            p2.abort();
            p3.abort()
        }
    }

    pub fn clear(&self) {
        self.store.clear();
    }

    pub async fn schedule(
        &self,
        task: &Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let erased = task.as_erased();
        let id = C::TaskIdentifier::generate();
        if let Some(channel) = &*self.instruction_channel.read().await {
            append_scheduler_handler::<C>(&erased, id.clone(), channel.clone()).await;
        }

        let trigger = erased.trigger().clone();

        self.store.store(&id, erased)?;
        assign_to_trigger_worker::<C>(trigger, &id, self.trigger_workers.as_ref()).await;

        Ok(id)
    }

    pub fn cancel(&self, idx: &C::TaskIdentifier) {
        self.store.remove(idx);
    }

    pub fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.store.exists(idx)
    }

    pub async fn has_started(&self) -> bool {
        self.process.read().await.is_some()
    }
}
