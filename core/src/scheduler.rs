#[allow(missing_docs)]
pub mod task_dispatcher;  // skipcq: RS-D1001

#[allow(missing_docs)]
pub mod task_store;  // skipcq: RS-D1001

use crate::clock::*;
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::DefaultSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::Task;
use once_cell::sync::Lazy;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

/// The default scheduler, it uses all the provided default components to build the scheduler.
/// Due to non-backend storage and system clock. This should **NOT** be used over
/// a different built scheduler
pub static CHRONOGRAPHER_SCHEDULER: Lazy<Arc<Scheduler>> =
    Lazy::new(|| Arc::new(Scheduler::builder().build()));

/// This is the builder configs to use for building a [`Scheduler`] instance.
/// By itself it should not be used, and it resides in [`Scheduler::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler))]
pub struct SchedulerConfig {
    /// The [`SchedulerTaskDispatcher`] for handling the execution of tasks. They are the
    /// mechanisms that drive load balancing, priority execution and so on...
    ///
    /// # Default Value
    /// Every scheduler uses as default value [`DefaultTaskDispatcher::default_configs()`]
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`DefaultTaskDispatcher`]
    /// - [`SchedulerTaskDispatcher`]
    /// - [`Scheduler`]
    #[builder(
        default = Arc::new(DefaultTaskDispatcher::default()),
        setter(transform = |std: impl SchedulerTaskDispatcher + 'static| Arc::new(std) as Arc<dyn SchedulerTaskDispatcher>),
    )]
    dispatcher: Arc<dyn SchedulerTaskDispatcher>,

    /// The [`SchedulerTaskStore`] for handling the storage of tasks. They are the
    /// mechanisms that drive backend storing, the retrieval of the earliest task and so on...
    ///
    /// # Default Value
    /// Every scheduler uses as default value [`PersistentDefaultTaskStore::new()`]. For simple
    /// demos and examples, this is fine for larger scale applications, backend storage mechanisms
    /// should be preferred to ensure tasks never fail (even when everything else fails)
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`PersistentDefaultTaskStore`]
    /// - [`SchedulerTaskStore`]
    /// - [`Scheduler`]
    #[builder(
        default = DefaultSchedulerTaskStore::ephemeral(),
        setter(transform = |std: impl SchedulerTaskStore + 'static| Arc::new(std) as Arc<dyn SchedulerTaskStore>),
    )]
    store: Arc<dyn SchedulerTaskStore>,

    /// The [`SchedulerClock`] for handling the idling of tasks and getting the present time.
    ///
    /// # Default Value
    /// Every scheduler uses as default value [`SystemClock`]. While for most cases, this is fine,
    /// when it comes to unit testing, stress-testing simulations, [`VirtualClock`] should be preferred
    /// as it allows explicit advancing of time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`SystemClock`]
    /// - [`VirtualClock`]
    /// - [`SchedulerClock`]
    /// - [`Scheduler`]
    #[builder(
        default = Arc::new(SystemClock),
        setter(transform = |clock: impl SchedulerClock + 'static| Arc::new(clock) as Arc<dyn SchedulerClock>),
    )]
    clock: Arc<dyn SchedulerClock>,
}

impl From<SchedulerConfig> for Scheduler {
    fn from(config: SchedulerConfig) -> Self {
        let (schedule_tx, schedule_rx) = broadcast::channel(16);

        Self {
            dispatcher: config.dispatcher,
            store: config.store,
            clock: config.clock,
            process: Mutex::new(None),
            schedule_tx: Arc::new(schedule_tx),
            schedule_rx: Arc::new(Mutex::new(schedule_rx)),
            notifier: Arc::new(tokio::sync::Notify::new()),
        }
    }
}

type ArcSchedulerTX = Arc<broadcast::Sender<usize>>;
type ArcSchedulerRX = Arc<Mutex<broadcast::Receiver<usize>>>;

/// [`Scheduler`] is the instance that hosts all the three composites those being:
///
/// - [`SchedulerTaskDispatcher`] for handling the execution of one and multiple tasks.
/// - [`SchedulerTaskStore`] for handling the storage of those tasks til they execute.
/// - [`SchedulerClock`] for handling the idling and getting the present time.
///
/// In addition, it handles the main scheduling loop which consists of in a nutshell:
/// 1. Retrieving the earliest task.
/// 2. Idling till the earliest task's target time is reached.
/// 3. Checking if the task still exists, if not then skip it.
/// 4. Dispatches the task to the [`SchedulerTaskDispatcher`] for execution.
/// 5. After finishing, the [`SchedulerTaskDispatcher`] notifies the
///    scheduler to reschedule the same task.
/// 6. Repeats for all the tasks.
///
/// # Constructor(s)
/// If one wishes to construct their own [`Scheduler`], they may do so via [`Scheduler::builder`],
/// alternatively, for simple demos and examples, it may be preferred to use the default provided
/// scheduler, that being [`CHRONOGRAPHER_SCHEDULER`]
///
/// # Implementation Detail(s)
/// The reason the [`Scheduler`] is a struct and not a trait is due to the fact that the loop,
/// the handling of tasks, the abortion of the scheduler and so on, do not differ per implementation.
/// As such, for convenience's sake, it is therefore a struct.
///
/// This does not mean it is not extensible, quite the contrary, as these three
/// composites define how the scheduler should work
///
/// # Trait Implementation(s)
/// The [`Scheduler`] implements the [`Debug`] trait which shows the debug output of all the composites
///
/// # Example
/// ```ignore
/// use chronographer_core::scheduler::Scheduler;
///
/// let scheduler = Scheduler::builder()
///     .clock(MY_CLOCK)
///     .store(MY_STORE)
///     .dispatcher(MY_DISPATCHER)
///     .build();
///
/// let idx = scheduler.schedule(MY_ARC_TASK_1).await; // Schedule with Arc value
/// let idx2 = scheduler.schedule_owned(MY_TASK_2).await; // Schedules with owned value
///
/// assert!(scheduler.exists(idx).await) // Checks if an ID exists
/// scheduler.cancel(idx).await;
///
/// scheduler.start().await; // Start the scheduler with the provided tasks
/// ```
///
/// # See Also
/// - [`CHRONOGRAPHER_SCHEDULER`]
/// - [`SchedulerTaskDispatcher`]
/// - [`SchedulerTaskStore`]
/// - [`SchedulerClock`]
pub struct Scheduler {
    dispatcher: Arc<dyn SchedulerTaskDispatcher>,
    store: Arc<dyn SchedulerTaskStore>,
    clock: Arc<dyn SchedulerClock>,
    process: Mutex<Option<JoinHandle<()>>>,
    schedule_tx: ArcSchedulerTX,
    schedule_rx: ArcSchedulerRX,
    notifier: Arc<tokio::sync::Notify>,
}

impl Debug for Scheduler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scheduler")
            .field("dispatcher", &self.dispatcher)
            .field("store", &self.store)
            .field("clock", &self.clock)
            .finish()
    }
}

impl Scheduler {
    /// Constructs a scheduler builder. Which is used for supplying
    /// various composites to then construct a [`Scheduler`], for
    /// simple enough demos and examples, it may be preferred to use
    /// the default provided [`CHRONOGRAPHER_SCHEDULER`]
    ///
    /// # Returns
    /// The [`SchedulerConfigBuilder`] builder for constructing the [`Scheduler`]
    ///
    /// # See Also
    /// - [`CHRONOGRAPHER_SCHEDULER`]
    /// - [`Scheduler`]
    /// - [`SchedulerConfigBuilder`]
    pub fn builder() -> SchedulerConfigBuilder {
        SchedulerConfig::builder()
    }

    /// Starts the scheduler, if the scheduler has already started, this method
    /// does nothing. The scheduler can be aborted if one wishes via [`Scheduler::abort`] and
    /// one can check if the scheduler has started via [`Scheduler::has_started`]
    ///
    /// # See Also
    /// - [`Scheduler`]
    /// - [`Scheduler::abort`]
    /// - [`Scheduler::has_started`]
    pub async fn start(&self) {
        if self.process.lock().await.is_some() {
            return;
        }
        let store_clone = self.store.clone();
        let clock_clone = self.clock.clone();
        let dispatcher_clone = self.dispatcher.clone();
        let scheduler_send = self.schedule_tx.clone();
        let scheduler_receive = self.schedule_rx.clone();
        let notifier = self.notifier.clone();
        *self.process.lock().await = Some(tokio::spawn(async move {
            let double_clock_clone = clock_clone.clone();
            let double_store_clone = store_clone.clone();
            let double_notifier_clone = notifier.clone();
            tokio::spawn(async move {
                while let Ok(idx) = scheduler_receive.lock().await.recv().await {
                    // This is the task dispatcher's duty to return the correct index.
                    // I am aware doing ``.unwrap()`` is an antipattern
                    let task = double_store_clone.get(&idx).await.unwrap();
                    if let Some(max_runs) = task.max_runs()
                        && task.runs() >= max_runs.get()
                    {
                        continue;
                    }
                    double_store_clone
                        .reschedule(double_clock_clone.clone(), &idx)
                        .await;
                    double_notifier_clone.notify_waiters();
                }
            });

            loop {
                if let Some((task, time, idx)) = store_clone.retrieve().await {
                    tokio::select! {
                        _ = clock_clone.idle_to(time) => {
                            store_clone.pop().await;
                            if !store_clone.exists(&idx).await { continue; }
                            dispatcher_clone.clone()
                                .dispatch(scheduler_send.clone(), task, idx)
                                .await;
                            continue;
                        }

                        _ = notifier.notified() => {
                            continue;
                        }
                    }
                }
            }
        }))
    }

    /// Aborts the scheduler, it acts like pausing the task, i.e. It doesn't clear any remaining
    /// tasks, in order to clear them as well, one should also use [`Scheduler::clear`]. If the scheduler
    /// hasn't started yet, this method does nothing, in this case it should be started via [`Scheduler::start`]
    /// or when trying to abort. Check via [`Scheduler::has_started`] and handle things from there
    ///
    /// # See Also
    /// - [`Scheduler::clear`]
    /// - [`Scheduler::start`]
    /// - [`Scheduler::has_started`]
    pub async fn abort(&self) {
        let process = self.process.lock().await.take();
        if let Some(p) = process {
            p.abort();
        }
    }

    /// Clears all [`Task`]s the scheduler has in store. This acts as
    /// a wrapper around the method [`SchedulerTaskStore::clear`].
    /// This method acts as a wrapper around [`SchedulerTaskStore`]
    ///
    /// # See Also
    /// - [`SchedulerTaskStore`]
    /// - [`Scheduler`]
    /// - [`Task`]
    pub async fn clear(&self) {
        self.store.clear().await;
    }

    /// Schedules a [`Task`] to run on the [`Scheduler`], if one
    /// wishes to fully schedule an owned version, then there is a
    /// convenience method of [`Scheduler::schedule_owned`] that is
    /// identical. This method acts more as a wrapper around the [`SchedulerTaskStore`]
    ///
    /// # Arguments
    /// It accepts a ``Arc<Task>``, which is non-owned. As such, this
    /// method is useful for when you need in other places the task and more so
    /// the task doesn't act as a one-off
    ///
    /// # Returns
    /// The index, which is used by some methods to refer to the task specifically (as opposed
    /// to having the full owned or non-owned task). Some of those are [`Scheduler::cancel`] and
    /// [`Scheduler::exists`]
    ///
    /// # See Also
    /// - [`Scheduler::exists`]
    /// - [`Scheduler::cancel`]
    /// - [`Scheduler::schedule_owned`]
    /// - [`SchedulerTaskStore`]
    /// - [`Task`]
    pub async fn schedule(&self, task: Arc<Task>) -> usize {
        self.store.store(self.clock.clone(), task).await
    }

    /// Schedules an owned [`Task`] to run on the [`Scheduler`], if one wishes to schedule
    /// a non-owned version (wrapped in an ``Arc``), then there is the method [`Scheduler::schedule`]
    /// which under the hood this method uses. This method acts more as a wrapper around the [`SchedulerTaskStore`]
    ///
    /// # Arguments
    /// It accepts a [`Task`] which is owned, as such this method is useful when you don't need the
    /// task in other places and the task more so acts as a one-off
    ///
    /// # Returns
    /// The index, which is used by some methods to refer to the task specifically (as opposed
    /// to having the full owned or non-owned task). Some of those are [`Scheduler::cancel`] and
    /// [`Scheduler::exists`]
    ///
    /// # See Also
    /// - [`Scheduler::exists`]
    /// - [`Scheduler::cancel`]
    /// - [`Scheduler::schedule_owned`]
    /// - [`SchedulerTaskStore`]
    /// - [`Task`]
    pub async fn schedule_owned(&self, task: Task) -> usize {
        self.schedule(Arc::new(task)).await
    }

    /// Cancels a [`Task`] via a provided index, when canceled a task will never be rescheduled and when
    /// it is attempted to run, it will be skipped. Depending on the [`SchedulerTaskStore`] supplied,
    /// if the index is invalid, then it may be skipped or something else may happen, as such it
    /// is advised to check the documentation of it
    ///
    /// # Usage Note(s)
    /// If the task is running while its being canceled, it has no effect on skipping the current
    /// instance running but more so any future schedules of this instance
    ///
    /// # Arguments
    /// The index that corresponds to the task, before calling the method, ensure the task exists via
    /// [`Scheduler::exists`] method and handle in your own in the case where it doesn't exist
    ///
    /// # See Also
    /// - [`SchedulerTaskStore`]
    /// - [`Scheduler::exists`]
    /// - [`Task`]
    pub async fn cancel(&self, idx: &usize) {
        self.store.remove(idx).await;
    }

    /// Checks if a [`Task`] exists based on an index, this method acts more
    /// as a wrapper around the [`SchedulerTaskStore`]
    ///
    /// # Arguments
    /// The index that may or may not be invalid, depending on the [`SchedulerTaskStore`]
    /// implementation supplied to the [`Scheduler`]
    ///
    /// # Returns
    /// a boolean value indicating if the task exists based on the index, if
    ///  true, then the task exists otherwise the task doesn't exist
    ///
    /// # See Also
    /// - [`Scheduler`]
    /// - [`SchedulerTaskStore`]
    /// - [`Task`]
    pub async fn exists(&self, idx: &usize) -> bool {
        self.store.exists(idx).await
    }

    /// Checks if the [`Scheduler`] has started
    ///
    /// # Returns
    /// The boolean value indicating if the scheduler has started or not, true if the
    /// scheduler has already started and false if it hasn't been started
    ///
    /// # See Also
    /// - [`Scheduler::start`]
    /// - [`Scheduler::abort`]
    pub async fn has_started(&self) -> bool {
        self.process.lock().await.is_some()
    }
}
