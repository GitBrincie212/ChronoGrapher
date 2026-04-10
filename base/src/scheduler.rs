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

pub type SchedulerKey<C> = <<C as SchedulerConfig>::SchedulerTaskStore as SchedulerTaskStore<C>>::Key;

#[derive(Debug)]
pub enum SchedulerWork {
    Trigger,
    Dispatch
}

pub(crate) struct SchedulerWorker<C: SchedulerConfig> {
    pub queue: SegQueue<(SchedulerKey<C>, SchedulerWork)>,
    pub notify: Arc<Notify>,
}

impl<C: SchedulerConfig> SchedulerWorker<C> {
    #[inline(always)]
    pub(crate) fn spawn_dispatch(&self, identifier: SchedulerKey<C>) {
        self.queue.push((identifier, SchedulerWork::Dispatch));
        self.notify.notify_one();
    }

    #[inline(always)]
    pub(crate) fn spawn_trigger(&self, identifier: SchedulerKey<C>) {
        self.queue.push((identifier, SchedulerWork::Trigger));
        self.notify.notify_one();
    }
}

pub(crate) type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);

pub type DefaultScheduler<E> = Scheduler<DefaultSchedulerConfig<E>>;

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

#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<C>))]
pub struct SchedulerInitConfig<C: SchedulerConfig> {
    dispatcher: C::SchedulerTaskDispatcher,
    store: C::SchedulerTaskStore,
    engine: C::SchedulerEngine,

    #[builder(default = 64)]
    workers: usize,

    #[builder(default = FailoverPolicy::default())]
    failover_policy: FailoverPolicy
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
            process: Arc::new(RwLock::new(Vec::new())),
            workers: Arc::new(workers),
            instruction_queue: Arc::new((SegQueue::<SchedulerHandlePayload>::new(), Notify::new())),
            failover_policy: config.failover_policy,
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: Arc<RwLock<Vec<JoinHandle<()>>>>,
    workers: Arc<Vec<SchedulerWorker<C>>>,
    instruction_queue: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
    failover_policy: FailoverPolicy,
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
async fn apply_failover<C: SchedulerConfig>(
    failover_policy: FailoverPolicy,
    key: &SchedulerKey<C>,
    worker: &SchedulerWorker<C>,
    work: SchedulerWork,
    store: &Arc<C::SchedulerTaskStore>,
    process: &Arc<RwLock<Vec<JoinHandle<()>>>>,
) {
    match failover_policy {
        FailoverPolicy::Keep => {
            worker.queue.push((key.clone(), work))
        }

        FailoverPolicy::Terminate => {}

        FailoverPolicy::Deallocate => {
            store.remove(&key)
        },

        FailoverPolicy::ShutdownScheduler => {
            let mut lock = process.write().await;
            let drained = lock.drain(..);
            for handle in drained {
                handle.abort();
            }
        }
    }
}

#[inline(always)]
fn spawn_task<C: SchedulerConfig>(
    key: SchedulerKey<C>,
    dispatch_workers: &Vec<SchedulerWorker<C>>
) {
    let idx = key.clone().into() & (dispatch_workers.len() - 1);
    dispatch_workers[idx].spawn_dispatch(key);
}

impl<C: SchedulerConfig> Scheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }

    pub async fn start(&self) {
        if self.has_started().await {
            return;
        }

        let engine_clone = self.engine.clone();
        let store_clone = self.store.clone();
        let dispatcher_clone = self.dispatcher.clone();

        join!(
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init()
        );

        let mut lock = self.process.write().await;
        for idx in 0..self.workers.len() {
            let workers = self.workers.clone();
            let store_clone = store_clone.clone();
            let dispatcher_clone = dispatcher_clone.clone();
            let engine_clone = engine_clone.clone();
            let worker_len = workers.len();
            let policy = self.failover_policy.clone();
            let processes = self.process.clone();
            let handle = tokio::spawn(async move {
                loop {
                    let mut pointing = idx;
                    for _ in 0..worker_len {
                        while let Some((key, work_type)) = workers[pointing].queue.pop()
                            && let Some(task) = store_clone.get(&key)
                        {
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
                                            apply_failover::<C>(
                                                policy, &key, &workers[pointing], work_type,
                                                &store_clone, &processes
                                            ).await;
                                            continue;
                                        }
                                    };

                                    match engine_clone.schedule(&key, time).await {
                                        Ok(()) => {}
                                        Err(err) => {
                                            eprintln!("Schedule error from SchedulerEngine: {:?}", err);
                                            apply_failover::<C>(
                                                policy, &key, &workers[pointing], work_type,
                                                &store_clone, &processes
                                            ).await;
                                        }
                                    }

                                    continue;
                                }

                                SchedulerWork::Dispatch => {
                                    let result = dispatcher_clone.dispatch(&key, task).await;
                                    match result {
                                        Ok(()) => {
                                            workers[pointing].spawn_trigger(key.clone())
                                        }

                                        Err(err) => {
                                            eprintln!(
                                                "Scheduler engine received an error for Task with identifier ({:?}):\n\t {:?}",
                                                key, err
                                            );
                                            apply_failover::<C>(
                                                policy, &key, &workers[pointing], work_type,
                                                &store_clone, &processes
                                            ).await;
                                        }
                                    }

                                    continue;
                                }
                            }
                        }

                        pointing = fastrand::usize(..worker_len);
                    }

                    workers[idx].notify.notified().await;
                }
            });

            lock.push(handle);
        }

        lock.push(tokio::spawn(
            main_loop_logic::<C>(
                &engine_clone,
                &self.workers
            )
        ));

        lock.push(tokio::spawn(
            scheduler_handle_instructions_logic::<C>(
                &self.instruction_queue,
                &dispatcher_clone,
                &store_clone,
                &self.workers
            ),
        ));
    }

    pub async fn abort(&self) {
        let mut lock = self.process.write().await;
        let handles = lock.drain(..);
        for handle in handles {
            handle.abort();
        }
    }

    pub fn clear(&self) {
        self.store.clear();
    }

    pub async fn schedule(
        &self,
        task: Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<SchedulerKey<C>, Box<dyn Error + Send + Sync>> {
        let erased = Arc::new(task.into_erased());
        let key = self.store.store(erased.clone())?;
        append_scheduler_handler::<C>(key.clone(), &erased, self.instruction_queue.clone()).await;
        assign_to_trigger_worker::<C>(key.clone(), self.workers.as_ref());

        Ok(key)
    }

    pub fn remove(&self, key: &SchedulerKey<C>) {
        self.store.remove(key);
    }

    pub fn exists(&self, key: &SchedulerKey<C>) -> bool {
        self.store.exists(key)
    }

    pub async fn has_started(&self) -> bool {
        !self.process.read().await.is_empty()
    }
}