pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001
mod utils; // skipcq: RS-D1001

use std::num::NonZeroUsize;
use crate::errors::TaskError;
use crate::scheduler::clock::*;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{TaskFrame, TaskHandle, TaskRef, TaskTrigger};
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
    Dispatch,
    Trigger,
    Instruction
}

pub(crate) struct SchedulerWorker<C: SchedulerConfig> {
    pub queue: SegQueue<(TaskHandle<C>, SchedulerWork)>,
    pub notify: Arc<Notify>,
}

impl<C: SchedulerConfig> SchedulerWorker<C> {
    #[inline(always)]
    pub(crate) fn spawn_dispatch(&self, handle: TaskHandle<C>) {
        self.queue.push((handle, SchedulerWork::Dispatch));
        self.notify.notify_one();
    }

    #[inline(always)]
    pub(crate) fn spawn_trigger(&self, handle: TaskHandle<C>) {
        self.queue.push((handle, SchedulerWork::Trigger));
        self.notify.notify_one();
    }

    #[inline(always)]
    pub(crate) fn spawn_instruction(&self, handle: TaskHandle<C>) {
        self.queue.push((handle, SchedulerWork::Instruction));
        self.notify.notify_one();
    }
}

#[inline(always)]
pub(crate) async fn do_trigger_work<C: SchedulerConfig>(handle: TaskHandle<C>, engine: &C::SchedulerEngine) {
    if let Some(trigger) = handle.trigger().await  {
        let now = engine.clock().now();
        let time = match trigger.trigger(now).await {
            Ok(time) => time,
            Err(err) => {
                eprintln!("Computation error from TaskTrigger: {:?}", err);
                handle.cancel().await;
                return;
            }
        };

        match engine.schedule(handle.clone(), time).await {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Schedule error from SchedulerEngine: {:?}", err);
                handle.cancel().await;
            }
        }
    }
}

#[inline(always)]
pub(crate) async fn do_dispatch_work<C: SchedulerConfig>(
    handle: TaskHandle<C>,
    engine: &C::SchedulerEngine,
    dispatcher: &C::SchedulerTaskDispatcher
) {
    let result = dispatcher.dispatch(&handle).await;
    match result {
        Ok(()) => {
            do_trigger_work(handle, engine).await;
        }

        Err(err) => {
            eprintln!("Scheduler engine received an error:\n\t {err:?}");
        }
    }
}

pub(crate) type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);
pub(crate) type ReschedulePayload<C> = (
    TaskHandle<C>,
    Option<<C as SchedulerConfig>::TaskError>
);

pub type DefaultScheduler<E = Box<dyn TaskError>> = Scheduler<DefaultSchedulerConfig<E>>;

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
    type SchedulerTaskDispatcher = DefaultTaskDispatcher;
    type SchedulerEngine = DefaultSchedulerEngine<Self>;
    type SchedulerClock = ProgressiveClock;
}

#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<T>))]
pub struct SchedulerInitConfig<T: SchedulerConfig> {
    dispatcher: T::SchedulerTaskDispatcher,
    store: T::SchedulerTaskStore,
    engine: T::SchedulerEngine,

    #[builder(default = (std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::MIN)
            .get() * 4).next_power_of_two()
    )]
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
            process: RwLock::new(Vec::new()),
            workers: Arc::new(workers),
            instruction_queue: Arc::new((SegQueue::<SchedulerHandlePayload>::new(), Notify::new())),
        }
    }
}

pub struct Scheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: RwLock<Vec<JoinHandle<()>>>,
    workers: Arc<Vec<SchedulerWorker<C>>>,
    instruction_queue: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
}

impl<C> Default for Scheduler<C>
where
    C: SchedulerConfig<
        SchedulerTaskStore: Default,
        SchedulerTaskDispatcher: Default,
        SchedulerEngine: Default,
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

impl<C: SchedulerConfig> Scheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }

    pub async fn start(&self) {
        let process_lock = self.process.read().await;
        if !process_lock.is_empty() {
            return;
        }

        drop(process_lock);

        let engine_clone = self.engine.clone();
        let dispatcher_clone = self.dispatcher.clone();

        join!(
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init()
        );

        let mut jobs = Vec::with_capacity(self.workers.len());
        for idx in 0..self.workers.len() {
            let workers = self.workers.clone();
            let dispatcher_clone = dispatcher_clone.clone();
            let engine_clone = engine_clone.clone();
            let worker_len = workers.len();
            let job = tokio::spawn(async move {
                loop {
                    let mut pointing = idx;
                    for _ in 0..worker_len {
                        let mut should_continue = true;
                        while let Some((handle, work_type)) = workers[pointing].queue.pop()
                            && should_continue && handle.is_valid()
                        {
                            should_continue = pointing == idx;
                            match work_type {
                                SchedulerWork::Trigger => {
                                    do_trigger_work::<C>(handle, engine_clone.as_ref()).await;
                                }

                                SchedulerWork::Dispatch => {
                                    do_dispatch_work::<C>(handle, engine_clone.as_ref(), dispatcher_clone.as_ref()).await;
                                }

                                SchedulerWork::Instruction => {}
                            }
                        }

                        pointing = fastrand::usize(..worker_len);
                    }

                    loop {
                        tokio::select! {
                            tasks = engine_clone.retrieve() => {
                                for handle in tasks {
                                    assign_dispatching_to_worker(handle, workers.as_ref());
                                }
                            }

                            _ = workers[idx].notify.notified() => {
                                break
                            }
                        }
                    }
                }
            });

            jobs.push(job);
        }

        *self.process.write().await = jobs;
    }

    pub async fn abort(&self) {
        let mut lock = self.process.write().await;
        for process in lock.drain(..) {
            process.abort();
        }
    }

    pub async fn clear(&self) {
        join!(
            self.store.clear(),
            self.engine.clear()
        );
    }

    pub async fn schedule(
        &self,
        trigger: impl TaskTrigger,
        frame: impl TaskFrame<Error = C::TaskError>
    ) -> Result<TaskHandle<C>, Box<dyn Error + Send + Sync>> {
        let incomplete_handle = self.store.allocate(trigger, frame).await?;
        let handle = TaskHandle::<C>::new(
            Arc::downgrade(&self.store),
            Arc::downgrade(&self.dispatcher),
            Arc::downgrade(&self.engine),
            Arc::downgrade(&self.workers),
            incomplete_handle
        );

        Ok(handle)
    }

    pub async fn has_started(&self) -> bool {
        !self.process.read().await.is_empty()
    }
}
