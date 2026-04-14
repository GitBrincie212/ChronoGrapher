use std::error::Error;
use std::sync::Arc;
use crossbeam::queue::SegQueue;
use tokio::join;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use crate::errors::TaskError;
use crate::scheduler::{DefaultSchedulerConfig, FailoverPolicy, Scheduler, SchedulerConfig, SchedulerHandlePayload, SchedulerKey};
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::scheduler::impls::utils::*;
use crate::task::{Task, TaskFrame, TaskTrigger};

pub type DefaultScheduler<E> = LiveScheduler<DefaultSchedulerConfig<E>>;

#[derive(Debug)]
#[repr(u8)]
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

#[derive(TypedBuilder)]
#[builder(build_method(into = LiveScheduler<C>))]
pub struct SchedulerInitConfig<C: SchedulerConfig> {
    dispatcher: C::SchedulerTaskDispatcher,
    store: C::SchedulerTaskStore,
    engine: C::SchedulerEngine,

    #[builder(default = 64)]
    workers: usize,

    #[builder(default = FailoverPolicy::default())]
    failover_policy: FailoverPolicy
}

impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for LiveScheduler<C> {
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
            process: Arc::new(parking_lot::RwLock::new(Vec::new())),
            workers: Arc::new(workers),
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
    workers: Arc<Vec<SchedulerWorker<C>>>,
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
    worker: &SchedulerWorker<C>,
    work: SchedulerWork,
    store: &Arc<C::SchedulerTaskStore>,
    process: &Arc<parking_lot::RwLock<Vec<JoinHandle<()>>>>,
) {
    match failover_policy {
        FailoverPolicy::Keep => {
            worker.queue.push((key.clone(), work))
        }

        FailoverPolicy::Terminate => {}

        FailoverPolicy::Deallocate => {
            store.remove(key)
        },

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
    workers: Arc<Vec<SchedulerWorker<C>>>,
    idx: usize,
    worker_len: usize,
    store_clone: Arc<C::SchedulerTaskStore>,
    engine_clone: Arc<C::SchedulerEngine>,
    dispatcher_clone: Arc<C::SchedulerTaskDispatcher>,
    policy: FailoverPolicy,
    processes: Arc<parking_lot::RwLock<Vec<JoinHandle<()>>>>,
) {
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
        for idx in 0..self.workers.len() {
            let handle = tokio::spawn(start_worker_process(
                self.workers.clone(),
                idx,
                self.workers.len(),
                self.store.clone(),
                self.engine.clone(),
                self.dispatcher.clone(),
                self.failover_policy,
                self.process.clone()
            ));

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

    fn abort(&self) -> impl Future<Output = ()> + Send {
        let mut lock = self.process.write();

        let handles = lock.drain(..);
        for handle in handles {
            handle.abort();
        }

        std::future::ready(())
    }

    fn clear(&self) -> impl Future<Output = ()> + Send {
        std::future::ready(self.store.clear())
    }

    async fn schedule(
        &self,
        task: Task<impl TaskFrame<Error= C::TaskError>, impl TaskTrigger>,
    ) -> Result<Self::Handle, Box<dyn Error + Send + Sync>> {
        let erased = Arc::new(task.into_erased());
        let key = self.store.store(erased.clone())?;
        append_scheduler_handler::<C>(key.clone(), &erased, self.instruction_queue.clone()).await;
        assign_to_trigger_worker::<C>(key.clone(), self.workers.as_ref());

        Ok(key)
    }

    fn remove(&self, key: &Self::Handle) -> impl Future<Output = ()> + Send {
        std::future::ready(self.store.remove(key))
    }

    fn exists(&self, key: &SchedulerKey<C>) -> impl Future<Output = bool> + Send {
        std::future::ready(self.store.exists(key))
    }

    fn has_started(&self)  -> impl Future<Output = bool> + Send {
        std::future::ready(!self.process.read().is_empty())
    }
}