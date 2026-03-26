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
use crate::utils::{SnowflakeID, TaskIdentifier};
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use crossbeam::deque;
use crossbeam::queue::SegQueue;
use tokio::join;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

pub(crate) use crate::scheduler::utils::*;

pub enum UnifiedWork<C: SchedulerConfig>{
    TaskDispatch(C::TaskIdentifier),
    TaskTrigger(C::TaskIdentifier),
    MainLoopTick,
    Reschedule(C::TaskIdentifier, Option<C::TaskError>),
    Instruction(C::TaskIdentifier, SchedulerHandleInstructions)
}

pub(crate) struct WorkerPool<C: SchedulerConfig> {
    pub injector: crossbeam::deque::Injector<UnifiedWork<C>>,
    pub stealers: std::sync::Mutex<Vec<crossbeam::deque::Stealer<UnifiedWork<C>>>>,
    pub notify: Arc<Notify>,
}

impl<C: SchedulerConfig> WorkerPool<C> {
    #[inline(always)]
    pub(crate) fn spawn_dispatch(&self, identifier: C::TaskIdentifier) {
        self.injector.push(UnifiedWork::TaskDispatch(identifier));
        self.notify.notify_one();
    }

    #[inline(always)]
    pub(crate) fn spawn_trigger(&self, identifier: C::TaskIdentifier) {
        self.injector.push(UnifiedWork::TaskTrigger(identifier));
        self.notify.notify_one();
    }
}

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
    type TaskIdentifier = SnowflakeID;
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

    #[builder(default = 64)]
    workers: usize,
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for Scheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        let notifier = Arc::new(Notify::new());
        let pool = WorkerPool::<C> {
            injector: crossbeam::deque::Injector::new(),
            stealers: std::sync::Mutex::new(Vec::with_capacity(config.workers)),
            notify: notifier,
        };

        Self {
            engine: Arc::new(config.engine),
            store: Arc::new(config.store),
            dispatcher: Arc::new(config.dispatcher),
            reschedule_queue: Arc::new((SegQueue::new(), Notify::new())),
            instruction_queue: Arc::new((SegQueue::new(), Notify::new())),
            process: RwLock::new(None),
            pool: Arc::new(pool),
            workers: config.workers,
        }
    }
}

pub(crate) struct SchedulerProcess {
    workers: Vec<JoinHandle<()>>,
    scheduler_handle_instructions: JoinHandle<()>,
    reschedule_loop: JoinHandle<()>,
    main_loop: JoinHandle<()>,
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    reschedule_queue: Arc<(SegQueue<ReschedulePayload<C>>, Notify)>,
    instruction_queue: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
    process: RwLock<Option<SchedulerProcess>>,
    pool: Arc<WorkerPool<C>>,
    workers: usize,
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

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(
    id: C::TaskIdentifier,
    pool: &WorkerPool<C>,
    workers: usize,
) {
    let _ = id.as_usize() & (workers.saturating_sub(1));
    pool.spawn_dispatch(id);
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

        let mut worker_handles: Vec<JoinHandle<()>> = Vec::with_capacity(self.workers);
        for _ in 0..self.workers {
            let pool = self.pool.clone();
            let store_clone = store_clone.clone();
            let dispatcher_clone = dispatcher_clone.clone();
            let engine_clone = engine_clone.clone();
            let reschedule_queue = self.reschedule_queue.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let local = crossbeam::deque::Worker::new_fifo();
                let stealer = local.stealer();
                {
                    let mut stealers = pool.stealers.lock().expect("poisoned stealers mutex");
                    stealers.push(stealer);
                }

                let rt = tokio::runtime::Handle::current();
                rt.block_on(async move {
                    loop {
                        let work = loop {
                            if let Some(work) = local.pop() {
                                break Some(work);
                            }

                            match pool.injector.steal_batch_and_pop(&local) {
                                deque::Steal::Success(work) => break Some(work),
                                deque::Steal::Retry => continue,
                                deque::Steal::Empty => {}
                            }

                            let stealers_snapshot = {
                                pool.stealers
                                    .lock()
                                    .expect("poisoned stealers mutex")
                                    .clone()
                            };

                            let mut stolen = None;
                            for stealer in &stealers_snapshot {
                                match stealer.steal() {
                                    deque::Steal::Success(work) => {
                                        stolen = Some(work);
                                        break;
                                    }
                                    deque::Steal::Retry => {}
                                    deque::Steal::Empty => {}
                                }
                            }
                            if stolen.is_some() {
                                break stolen;
                            }

                            break None;
                        };

                        let Some(work) = work else {
                            pool.notify.notified().await;
                            continue;
                        };

                        match work {
                            UnifiedWork::TaskTrigger(id) => {
                                let Some(task) = store_clone.get(&id) else {
                                    continue;
                                };
                                let trigger = task.trigger();
                                let now = engine_clone.clock().now();

                                let time = match trigger.trigger(now).await {
                                    Ok(time) => time,
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
                            }

                            UnifiedWork::TaskDispatch(id) => {
                                let Some(task) = store_clone.get(&id) else {
                                    continue;
                                };
                                let result = dispatcher_clone.dispatch(&id, task).await;
                                reschedule_queue.0.push((id, result.err()));
                                reschedule_queue.1.notify_waiters();
                            }

                            UnifiedWork::MainLoopTick
                            | UnifiedWork::Reschedule(_, _)
                            | UnifiedWork::Instruction(_, _) => {}
                        }
                    }
                });
            });

            worker_handles.push(handle);
        }

        let reschedule_loop = tokio::spawn(
            reschedule_logic::<C>(
                &self.reschedule_queue,
                &self.pool,
                self.workers,
            )
        );

        let main_loop = tokio::spawn(
            main_loop_logic::<C>(
                &engine_clone,
                &self.pool,
                self.workers,
            )
        );

        let scheduler_handle_instructions = tokio::spawn(
            scheduler_handle_instructions_logic::<C>(
                &self.instruction_queue,
                &dispatcher_clone,
                &store_clone,
                &self.pool,
                self.workers,
            ),
        );

        *self.process.write().await = Some(SchedulerProcess {
            workers: worker_handles,
            scheduler_handle_instructions,
            reschedule_loop,
            main_loop,
        });
    }

    pub async fn abort(&self) {
        let process = self.process.write().await.take();
        if let Some(process) = process {
            for worker in process.workers {
                worker.abort();
            }
            process.scheduler_handle_instructions.abort();
            process.reschedule_loop.abort();
            process.main_loop.abort();
        }
    }

    pub fn clear(&self) {
        self.store.clear();
    }

    pub async fn schedule(
        &self,
        task: Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let erased = task.into_erased();
        let id = C::TaskIdentifier::generate();

        self.store.store(&id, erased)?;
        assign_to_trigger_worker::<C>(id.clone(), &self.pool, self.workers);

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
