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
use crossbeam::queue::SegQueue;
use tokio::join;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

pub(crate) use crate::scheduler::utils::*;

pub enum SchedulerWork {
    Trigger,
    Dispatch
}

pub(crate) struct SchedulerWorker<C: SchedulerConfig> {
    pub queue: SegQueue<(C::TaskIdentifier, SchedulerWork)>,
    pub notify: Arc<Notify>,
}

impl<C: SchedulerConfig> SchedulerWorker<C> {
    #[inline(always)]
    pub(crate) fn spawn_dispatch(&self, identifier: C::TaskIdentifier) {
        self.queue.push((identifier, SchedulerWork::Dispatch));
        self.notify.notify_one();
    }

    #[inline(always)]
    pub(crate) fn spawn_trigger(&self, identifier: C::TaskIdentifier) {
        self.queue.push((identifier, SchedulerWork::Dispatch));
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
        let mut workers = Vec::with_capacity(config.workers);
        let notifier = Arc::new(Notify::new());

        for _ in 0..config.workers {
            let worker = SchedulerWorker::<C> {
                queue: SegQueue::new(),
                notify: notifier.clone(),
            };
            workers.push(worker);
        }

        Self {
            engine: Arc::new(config.engine),
            store: Arc::new(config.store),
            dispatcher: Arc::new(config.dispatcher),
            process: RwLock::new(None),
            workers: Arc::new(workers),
            instruction_queue: Arc::new((SegQueue::<SchedulerHandlePayload>::new(), Notify::new())),
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: RwLock<Option<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>)>>,
    workers: Arc<Vec<SchedulerWorker<C>>>,
    instruction_queue: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
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
    dispatch_workers: &Vec<SchedulerWorker<C>>
) {
    let idx = id.as_usize() & (dispatch_workers.len() - 1);
    dispatch_workers[idx].spawn_dispatch(id);
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

        let reschedule_queue =
            Arc::new((SegQueue::<ReschedulePayload<C>>::new(), Notify::new()));

        for idx in 0..self.workers.len() {
            let workers = self.workers.clone();
            let store_clone = store_clone.clone();
            let dispatcher_clone = dispatcher_clone.clone();
            let engine_clone = engine_clone.clone();
            let reschedule_queue_clone = reschedule_queue.clone();
            let worker_len = workers.len();
            tokio::spawn(async move {
                let mut pointing = idx;
                for _ in 0..worker_len {
                    let mut should_continue = true;
                    while let Some((id, work_type)) = workers[pointing].queue.pop()
                        && should_continue {
                        if let Some(task) = store_clone.get(&id) {
                            should_continue = pointing == idx;
                            match work_type {
                                SchedulerWork::Trigger => {
                                    let trigger = task.trigger();
                                    let now = engine_clone.clock().now();

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

                                SchedulerWork::Dispatch => {
                                    let result = dispatcher_clone.dispatch(&id, task).await;
                                    reschedule_queue_clone.0.push((id, result.err()));
                                    reschedule_queue_clone.1.notify_waiters();
                                    continue;
                                }
                            }
                        }
                    }

                    pointing = fastrand::usize(..worker_len);
                }

                workers[idx].notify.notified().await;
            });
        }

        let reschedule_loop = tokio::spawn(
            reschedule_logic::<C>(
                &reschedule_queue,
                &self.workers
            )
        );

        let main_loop = tokio::spawn(
            main_loop_logic::<C>(
                &engine_clone,
                &self.workers
            )
        );

        let scheduler_handle_instructions = tokio::spawn(
            scheduler_handle_instructions_logic::<C>(
                &self.instruction_queue,
                &dispatcher_clone,
                &store_clone,
                &self.workers
            ),
        );

        *self.process.write().await = Some((
            scheduler_handle_instructions,
            reschedule_loop,
            main_loop
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
        task: Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let erased = task.into_erased();
        let id = C::TaskIdentifier::generate();

        append_scheduler_handler::<C>(&erased, id.clone(), self.instruction_queue.clone()).await;

        self.store.store(&id, erased)?;
        assign_to_trigger_worker::<C>(id.clone(), self.workers.as_ref());

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