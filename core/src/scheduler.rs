pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001
mod utils; // skipcq: RS-D1001

use crate::errors::TaskError;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{Task, TaskFrame, TaskTrigger};
use crate::utils::{DefaultTaskID, TaskIdentifier};
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use crossbeam::queue::SegQueue;
use tokio::join;
use tokio::sync::{Notify, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use typed_builder::TypedBuilder;

pub(crate) use crate::scheduler::utils::*;

pub(crate) type TriggerJobPayload<C> = (
    <C as SchedulerConfig>::TaskIdentifier,
    Arc<dyn TaskTrigger>,
);

pub(crate) type Worker<T> = Arc<(SegQueue<T>, Notify)>;
pub(crate) type TriggerJobWorker<C> = Worker<TriggerJobPayload<C>>;

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

    #[builder(default = 16)]
    trigger_workers: usize,

    #[builder(default = 64)]
    dispatch_workers: usize,
}

#[inline(always)]
fn create_worker<T>() -> Worker<T> {
    Arc::new((
        SegQueue::<T>::new(),
        Notify::new()
    ))
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for Scheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        let engine = Arc::new(config.engine);
        let store = Arc::new(config.store);
        let mut trigger_workers = Vec::with_capacity(config.trigger_workers);

        for _ in 0..config.trigger_workers {
            let worker = create_worker::<(C::TaskIdentifier, Arc<dyn TaskTrigger>)>();
            
            let engine_clone = engine.clone();
            let store_clone = store.clone();
            let worker_clone = worker.clone();
            tokio::spawn(async move {
                loop {
                    if let Some((id, trigger))
                        = worker_clone.0.pop()
                    {
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
                            }
                        }

                        continue;
                    }

                    worker_clone.1.notified().await;
                }


            });

            trigger_workers.push(worker);
        }

        Self {
            engine,
            store,
            dispatcher: Arc::new(config.dispatcher),
            process: RwLock::new(None),
            instruction_channel: RwLock::new(None),
            trigger_workers: Arc::new(trigger_workers),
            dispatch_workers: config.dispatch_workers,
            pre_schedule_queue: Arc::new(SegQueue::new()),
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: RwLock<Option<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>)>>,
    instruction_channel: RwLock<Option<tokio::sync::mpsc::Sender<SchedulerHandlePayload>>>,
    trigger_workers: Arc<Vec<TriggerJobWorker<C>>>,
    dispatch_workers: usize,
    pre_schedule_queue: Arc<SegQueue<C::TaskIdentifier>>,
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
    dispatch_workers: Arc<Vec<Worker<C::TaskIdentifier>>>
) {
    let idx = id.as_usize() & (dispatch_workers.len() - 1);
    dispatch_workers[idx].0.push(id);
    dispatch_workers[idx].1.notify_waiters();
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
            self.engine.init()
        );

        let mut dispatch_workers = Vec::with_capacity(self.dispatch_workers);

        let reschedule_queue =
            Arc::new((SegQueue::<ReschedulePayload<C>>::new(), Notify::new()));

        for _ in 0..self.dispatch_workers {
            let worker = create_worker::<C::TaskIdentifier>();

            let worker_clone = worker.clone();
            let store_clone = store_clone.clone();
            let dispatcher_clone = dispatcher_clone.clone();
            let reschedule_queue_clone = reschedule_queue.clone();
            tokio::spawn(async move {
                loop {
                    if let Some(id) = worker_clone.0.pop()
                        && let Some(task) = store_clone.get(&id)
                    {
                        let result = dispatcher_clone.dispatch(&id, task).await;
                        reschedule_queue_clone.0.push((id, result.err()));
                        reschedule_queue_clone.1.notify_waiters();
                        continue;
                    }

                    worker_clone.1.notified().await;
                }


            });

            dispatch_workers.push(worker);
        }

        let dispatch_workers = Arc::new(dispatch_workers);

        let (instruct_send, instruct_receive) =
            tokio::sync::mpsc::channel::<SchedulerHandlePayload>(1024);

        let workers = (2 * self.pre_schedule_queue.len()).isqrt().max(1);
        let mut js = JoinSet::new();
        for _ in 0..workers {
            let queue_clone = self.pre_schedule_queue.clone();
            let store_clone = self.store.clone();
            let instruct_send_clone = instruct_send.clone();
            js.spawn(async move {
                while let Some(id) = queue_clone.pop()
                    && let Some(task) = store_clone.get(&id) {
                    append_scheduler_handler::<C>(&task, id, instruct_send_clone.clone()).await;
                }
            });
        }

        js.join_all().await;

        *self.instruction_channel.write().await = Some(instruct_send);

        *self.process.write().await = Some((
            tokio::spawn(
                scheduler_handle_instructions_logic::<C>(
                    instruct_receive,
                    &dispatcher_clone,
                    &store_clone,
                    self.trigger_workers.clone(),
                    &dispatch_workers
                ),
            ),

            tokio::spawn(
                reschedule_logic::<C>(
                    &store_clone,
                    &reschedule_queue,
                    self.trigger_workers.clone()
                )
            ),

            tokio::spawn(
                main_loop_logic::<C>(
                    &engine_clone,
                    &dispatch_workers
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
        } else {
            self.pre_schedule_queue.push(id.clone());
        }

        let trigger = erased.trigger().clone();

        self.store.store(&id, erased)?;
        assign_to_trigger_worker::<C>(trigger, &id, self.trigger_workers.as_ref());

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
