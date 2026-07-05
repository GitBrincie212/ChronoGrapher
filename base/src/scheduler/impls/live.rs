use crate::errors::TaskError;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::impls::utils::*;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::scheduler::{
    DefaultSchedulerConfig, FailoverPolicy, Scheduler, SchedulerConfig, SchedulerHandlePayload,
    SchedulerKey,
};
use crate::task::{Task, TaskFrame};
use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use crossbeam::queue::SegQueue;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam::utils::CachePadded;
use tokio::join;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

pub type DefaultLiveScheduler<E> = LiveScheduler<DefaultSchedulerConfig<E>>;

#[cfg(feature = "anyhow")]
pub type DefaultLiveAnyhowScheduler = DefaultLiveScheduler<anyhow::Error>;

#[cfg(feature = "eyre")]
pub type DefaultLiveEyreScheduler = DefaultLiveScheduler<eyre::Error>;

#[derive(Debug)]
#[repr(u8)]
pub enum SchedulerWork {
    Trigger,
    Dispatch,
}

pub(crate) struct SchedulerWorkerHot<C: SchedulerConfig> {
    pub ingress: SegQueue<(SchedulerKey<C>, SchedulerWork)>,
    pub stealer: Stealer<(SchedulerKey<C>, SchedulerWork)>,
}

pub(crate) struct SchedulerWorkerCold<C: SchedulerConfig> {
    pub queue: parking_lot::Mutex<Option<Worker<(SchedulerKey<C>, SchedulerWork)>>>,
    pub notify: Arc<Notify>,
    pub pending: CachePadded<AtomicUsize>
}

#[inline(always)]
fn new_worker<C: SchedulerConfig>(notify: Arc<Notify>) -> (SchedulerWorkerHot<C>, SchedulerWorkerCold<C>) {
    let queue = Worker::new_fifo();
    let stealer = queue.stealer();

    (
        SchedulerWorkerHot {
            ingress: SegQueue::new(),
            stealer,
        },

        SchedulerWorkerCold {
            queue: parking_lot::Mutex::new(Some(queue)),
            notify,
            pending: CachePadded::new(AtomicUsize::new(0)),
        }
    )
}

#[derive(TypedBuilder)]
#[builder(build_method(into = LiveScheduler<C>))]
pub struct SchedulerInitConfig<C: SchedulerConfig> {
    dispatcher: C::SchedulerTaskDispatcher,
    store: C::SchedulerTaskStore,
    engine: C::SchedulerEngine,

    #[builder(default, setter(strip_option))]
    workers: Option<usize>,

    #[builder(default = FailoverPolicy::default())]
    failover_policy: FailoverPolicy,
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for LiveScheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        let workers = config.workers.unwrap_or_else(|| {
            let parallelism = std::thread::available_parallelism()
                .unwrap()
                .get();

            (parallelism * 4).next_power_of_two()
        });

        let mut cold_workers = Vec::with_capacity(workers);
        let mut hot_workers = Vec::with_capacity(workers);

        for _ in 0..workers {
            let notifier = Arc::new(Notify::new());
            let (hot_worker, cold_worker) = new_worker::<C>(notifier);
            cold_workers.push(CachePadded::new(cold_worker));
            hot_workers.push(CachePadded::new(hot_worker));
        }

        Self {
            engine: Arc::new(config.engine),
            store: Arc::new(config.store),
            dispatcher: Arc::new(config.dispatcher),
            process: Arc::new(parking_lot::RwLock::new(Vec::new())),

            cold_workers: Arc::new(cold_workers),
            hot_workers: Arc::new(hot_workers),
            worker_len: workers,

            global_queue: Arc::new(Injector::new()),
            instruction_queue: Arc::new((SegQueue::<SchedulerHandlePayload>::new(), Notify::new())),
            failover_policy: config.failover_policy,
        }
    }
}

pub struct LiveScheduler<C: SchedulerConfig> {
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: Arc<parking_lot::RwLock<Vec<JoinHandle<()>>>>,

    hot_workers: Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
    worker_len: usize,

    global_queue: Arc<Injector<(SchedulerKey<C>, SchedulerWork)>>,
    instruction_queue: Arc<(SegQueue<SchedulerHandlePayload>, Notify)>,
    failover_policy: FailoverPolicy,
}

impl<C> Default for LiveScheduler<C>
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
    global_queue: &Arc<Injector<(SchedulerKey<C>, SchedulerWork)>>,
    work: SchedulerWork,
    store: &Arc<C::SchedulerTaskStore>,
    process: &Arc<parking_lot::RwLock<Vec<JoinHandle<()>>>>,
) {
    match failover_policy {
        FailoverPolicy::Keep => {
            global_queue.push((key.clone(), work));
        }

        FailoverPolicy::Terminate => {}

        FailoverPolicy::Deallocate => store.remove(key),

        FailoverPolicy::ShutdownScheduler => {
            let mut lock = process.write();
            let drained = lock.drain(..);
            for handle in drained {
                handle.abort();
            }
        }
    }
}

#[inline(always)]
async fn start_worker_process<C: SchedulerConfig>(
    hot_workers: Arc<Vec<CachePadded<SchedulerWorkerHot<C>>>>,
    cold_workers: Arc<Vec<CachePadded<SchedulerWorkerCold<C>>>>,
    global_queue: Arc<Injector<(SchedulerKey<C>, SchedulerWork)>>,
    idx: usize,
    worker_len: usize,
    store_clone: Arc<C::SchedulerTaskStore>,
    engine_clone: Arc<C::SchedulerEngine>,
    dispatcher_clone: Arc<C::SchedulerTaskDispatcher>,
    policy: FailoverPolicy,
    processes: Arc<parking_lot::RwLock<Vec<JoinHandle<()>>>>,
) {
    let local_worker = {
        let mut lock = cold_workers[idx].queue.lock();
        lock.take().expect("worker queue was already taken")
    };

    loop {
        while let Some(work) = hot_workers[idx].ingress.pop() {
            local_worker.push(work);
        }

        while let Some((key, work_type)) = local_worker.pop() {
            if let Some(task) = store_clone.get(&key) {
                match work_type {
                    SchedulerWork::Trigger => {
                        let schedule = task.schedule();
                        let now = engine_clone.clock().now();

                        let time = match schedule.schedule(now).await {
                            Ok(time) => time,

                            Err(err) => {
                                eprintln!("Computation error from TaskTrigger: {:?}", err);
                                apply_failover::<C>(
                                    policy,
                                    &key,
                                    &global_queue,
                                    work_type,
                                    &store_clone,
                                    &processes,
                                )
                                .await;
                                continue;
                            }
                        };

                        match engine_clone.schedule(&key, time).await {
                            Ok(()) => {}

                            Err(err) => {
                                eprintln!("Schedule error from SchedulerEngine: {:?}", err);
                                apply_failover::<C>(
                                    policy,
                                    &key,
                                    &global_queue,
                                    work_type,
                                    &store_clone,
                                    &processes,
                                )
                                .await;
                            }
                        }
                    }

                    SchedulerWork::Dispatch => {
                        let result = dispatcher_clone.dispatch(&key, task).await;
                        match result {
                            Ok(()) => {
                                local_worker.push((key, SchedulerWork::Trigger));
                            }

                            Err(err) => {
                                eprintln!(
                                    "Scheduler engine received an error for Task with identifier ({:?}):\n\t {:?}",
                                    key, err
                                );
                                apply_failover::<C>(
                                    policy,
                                    &key,
                                    &global_queue,
                                    work_type,
                                    &store_clone,
                                    &processes,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }

        let mut found_work = false;
        let steal_attempts = worker_len.min(4);
        for _ in 0..steal_attempts {
            let victim = fastrand::usize(..worker_len);
            if victim == idx {
                continue;
            }

            match hot_workers[victim].stealer.steal_batch_and_pop(&local_worker) {
                Steal::Success(work) => {
                    local_worker.push(work);
                    found_work = true;
                    break;
                }
                Steal::Retry | Steal::Empty => {}
            }
        }

        if found_work {
            continue;
        }

        loop {
            match global_queue.steal_batch_and_pop(&local_worker) {
                Steal::Success(work) => {
                    local_worker.push(work);
                    found_work = true;
                    break;
                }
                Steal::Retry => continue,
                Steal::Empty => break,
            }
        }

        if found_work {
            continue;
        }


        if cold_workers[idx].pending.swap(0, Ordering::Relaxed) > 0 {
            continue;
        }

        cold_workers[idx].notify.notified().await;
    }
}

impl<C: SchedulerConfig> LiveScheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }
}

impl<C: SchedulerConfig> Scheduler<C> for LiveScheduler<C> {
    type Handle = SchedulerKey<C>;

    async fn start(&self) {
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

        let mut lock = self.process.write();
        for idx in 0..self.worker_len {
            let handle = tokio::spawn(start_worker_process(
                self.hot_workers.clone(),
                self.cold_workers.clone(),
                self.global_queue.clone(),
                idx,
                self.worker_len,
                self.store.clone(),
                self.engine.clone(),
                self.dispatcher.clone(),
                self.failover_policy,
                self.process.clone(),
            ));

            lock.push(handle);
        }

        lock.push(tokio::spawn(main_loop_logic::<C>(
            &engine_clone,
            &self.hot_workers,
            &self.cold_workers,
        )));

        lock.push(tokio::spawn(scheduler_handle_instructions_logic::<C>(
            &self.instruction_queue,
            &dispatcher_clone,
            &store_clone,
            &self.hot_workers,
            &self.cold_workers,
        )));
    }

    fn has_started(&self) -> impl Future<Output = bool> + Send {
        std::future::ready(!self.process.read().is_empty())
    }

    fn abort(&self) -> impl Future<Output = ()> + Send {
        let mut lock = self.process.write();

        let handles = lock.drain(..);
        for handle in handles {
            handle.abort();
        }

        std::future::ready(())
    }

    fn exists(&self, key: &SchedulerKey<C>) -> impl Future<Output = bool> + Send {
        std::future::ready(self.store.exists(key))
    }

    async fn schedule<T: TaskFrame<Args = (), Error = C::TaskError>>(
        &self,
        task: Task<T>,
    ) -> Result<Self::Handle, Box<dyn Error + Send + Sync>> {
        let erased = Arc::new(task.into_erased());
        let key = self.store.store(erased.clone())?;
        append_scheduler_handler::<C>(key.clone(), &erased, self.instruction_queue.clone()).await;
        assign_to_trigger_worker::<C>(key.clone(), &self.hot_workers, &self.cold_workers);

        Ok(key)
    }

    fn remove(&self, key: &Self::Handle) -> impl Future<Output = ()> + Send {
        std::future::ready(self.store.remove(key))
    }

    fn clear(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(self.store.clear())
    }
}
