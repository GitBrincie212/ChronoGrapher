pub mod clock;
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001 // skipcq: RS-D1001

use crate::scheduler::clock::*;
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::DefaultSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ScheduleStrategy, Task, TaskFrame, TaskSchedule};
use crate::utils::Timestamp;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;
use tokio::join;
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use uuid::Uuid;

/// The default scheduler's type alias, it uses all the provided default components
pub type DefaultScheduler = Scheduler<
    SystemTime,
    ProgressiveClock<SystemTime>,
    DefaultSchedulerTaskStore,
    DefaultTaskDispatcher,
>;

/// The default scheduler, it uses all the provided default components to build the scheduler.
/// Due to non-backend storage and system clock. This should **NOT** be used over
/// a different built scheduler
pub static CHRONOGRAPHER_SCHEDULER: LazyLock<Arc<DefaultScheduler>> = LazyLock::new(|| {
    Arc::new(
        Scheduler::builder()
            .store(DefaultSchedulerTaskStore::ephemeral())
            .clock(ProgressiveClock::<SystemTime>::default())
            .dispatcher(DefaultTaskDispatcher)
            .build(),
    )
});

/// This is the builder configs to use for building a [`Scheduler`] instance.
/// By itself it should not be used, and it resides in [`Scheduler::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<SupportedDateFormat, T1, T2, T3>))]
pub struct SchedulerConfig<
    SupportedDateFormat: Timestamp,
    T1: SchedulerClock<SupportedDateFormat>,
    T2: SchedulerTaskStore<SupportedDateFormat>,
    T3: SchedulerTaskDispatcher,
> {
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
    dispatcher: T3,

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
    store: T2,

    /// The [`SchedulerClock`] for handling the idling of tasks and getting the present time.
    ///
    /// # Default Value
    /// Every scheduler uses as default value [`ProgressiveClock`]. While for most cases, this is fine,
    /// when it comes to unit testing, stress-testing simulations, [`VirtualClock`] should be preferred
    /// as it allows explicit advancing of time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`ProgressiveClock`]
    /// - [`VirtualClock`]
    /// - [`SchedulerClock`]
    /// - [`Scheduler`]
    clock: T1,

    #[builder(default, setter(skip))]
    dateformat: PhantomData<SupportedDateFormat>,
}

impl<SupportedDateFormat, T1, T2, T3> From<SchedulerConfig<SupportedDateFormat, T1, T2, T3>>
    for Scheduler<SupportedDateFormat, T1, T2, T3>
where
    SupportedDateFormat: Timestamp,
    T1: SchedulerClock<SupportedDateFormat>,
    T2: SchedulerTaskStore<SupportedDateFormat>,
    T3: SchedulerTaskDispatcher,
{
    fn from(config: SchedulerConfig<SupportedDateFormat, T1, T2, T3>) -> Self {
        let (schedule_tx, schedule_rx) = broadcast::channel(16);

        Self {
            dispatcher: Arc::new(config.dispatcher),
            store: Arc::new(config.store),
            clock: Arc::new(config.clock),
            process: Mutex::new(None),
            schedule_tx: Arc::new(schedule_tx),
            schedule_rx: Arc::new(Mutex::new(schedule_rx)),
            notifier: Arc::new(tokio::sync::Notify::new()),
            _supported_date_format: PhantomData,
        }
    }
}

type ArcSchedulerTX<T> = Arc<broadcast::Sender<T>>;
type ArcSchedulerRX<T> = Arc<Mutex<broadcast::Receiver<T>>>;

pub struct RescheduleNotifier<T: 'static> {
    value: T,
    notify: Arc<broadcast::Sender<T>>,
}

impl<T: 'static> RescheduleNotifier<T> {
    pub fn notify(self) -> Result<usize, broadcast::error::SendError<T>> {
        self.notify.send(self.value)
    }
}

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
pub struct Scheduler<
    SupportedDateFormat: Timestamp,
    T1: SchedulerClock<SupportedDateFormat>,
    T2: SchedulerTaskStore<SupportedDateFormat>,
    T3: SchedulerTaskDispatcher,
> {
    clock: Arc<T1>,
    store: Arc<T2>,
    dispatcher: Arc<T3>,
    process: Mutex<Option<JoinHandle<()>>>,
    schedule_tx: ArcSchedulerTX<Uuid>,
    schedule_rx: ArcSchedulerRX<Uuid>,
    notifier: Arc<tokio::sync::Notify>,
    _supported_date_format: PhantomData<SupportedDateFormat>,
}

impl<SupportedDateFormat, T1, T2, T3> Scheduler<SupportedDateFormat, T1, T2, T3>
where
    SupportedDateFormat: Timestamp,
    T1: SchedulerClock<SupportedDateFormat>,
    T2: SchedulerTaskStore<SupportedDateFormat>,
    T3: SchedulerTaskDispatcher,
{
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
    pub fn builder() -> SchedulerConfigBuilder<SupportedDateFormat, T1, T2, T3> {
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
        let process_lock = self.process.lock().await;
        if process_lock.is_some() {
            return;
        }
        drop(process_lock);
        let store_clone = self.store.clone();
        let clock_clone = self.clock.clone();
        let dispatcher_clone = self.dispatcher.clone();
        let scheduler_send = self.schedule_tx.clone();
        let scheduler_receive = self.schedule_rx.clone();
        let notifier = self.notifier.clone();
        join!(self.store.init(), self.dispatcher.init());
        *self.process.lock().await = Some(tokio::spawn(async move {
            let double_clock_clone = clock_clone.clone();
            let double_store_clone = store_clone.clone();
            let double_notifier_clone = notifier.clone();
            tokio::spawn(async move {
                while let Ok(idx) = scheduler_receive.lock().await.recv().await {
                    if let Some(task) = double_store_clone.get(&idx).await {
                        if let Some(max_runs) = task.max_runs()
                            && task.runs() >= max_runs.get()
                        {
                            continue;
                        }
                        double_store_clone
                            .reschedule(&double_clock_clone, &idx)
                            .await;
                        double_notifier_clone.notify_waiters();
                    }
                }
            });

            loop {
                if let Some((task, time, idx)) = store_clone.retrieve().await {
                    tokio::select! {
                        _ = clock_clone.idle_to(time) => {
                            store_clone.pop().await;
                            if !store_clone.exists(&idx).await { continue; }
                            let sender = RescheduleNotifier {
                                value: idx,
                                notify: scheduler_send.clone()
                            };
                            dispatcher_clone
                                .dispatch(task, sender)
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
    pub async fn schedule(
        &self,
        task: &Task<impl TaskFrame, impl TaskSchedule, impl ScheduleStrategy>,
    ) -> Uuid {
        let erased = task.as_erased();
        let idx = self.store.store(&self.clock, erased).await;
        idx
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
    pub async fn cancel(&self, idx: &Uuid) {
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
    pub async fn exists(&self, idx: &Uuid) -> bool {
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
