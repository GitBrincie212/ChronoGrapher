pub mod schedule; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::Task;
pub use crate::task::trigger::schedule::calendar::TaskCalendarField;
pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
pub use crate::task::trigger::schedule::cron::TaskScheduleCron;
pub use crate::task::trigger::schedule::immediate::TaskScheduleImmediate;
pub use crate::task::trigger::schedule::interval::TaskScheduleInterval;
use async_trait::async_trait;
use std::any::Any;
use std::error::Error;
use std::time::SystemTime;

/// [`TaskTrigger`] is the main mechanism in which [`Tasks`](Task) schedule a future time (based on
/// a current one) to run, this time is handed to the "[`Scheduler`](crate::scheduler::Scheduler) Side"
/// for it to organize.
///
/// [`TaskTrigger`] may immediately hand out the future time (in this case, best use [`TaskSchedule`](schedule::TaskSchedule)
/// or notify at any other time the "Scheduler Side" about its future time to schedule to.
///
/// # Semantics
/// There are 2 arguments, the first is the "now" argument which utilizes [`SystemTime`] provided by Rust.
///
/// > **Important Note:** The value for the "now" argument is **NOT** the same as using [`SystemTime::now`],
/// the value is defined by which [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) is used
///
/// The second argument is a [`TriggerNotifier`] which is the main channel in which the [`TaskTrigger`]
/// sends its results back to "Scheduler Side".
///
/// There are two cases where [`TaskTrigger`] may error out **Errors During Initialization** are caused
/// when calling the [`TaskTrigger::trigger`] method.
///
/// Reasons in which a [`TaskTrigger`] may error out can be due to restricted access to the network (or a service),
/// I/O issues (for monitoring files) and anything else in-between.
///
/// Then there are **Errors During Computation**, these happen at a later stage, and they involve sending
/// the results via [`TriggerNotifier`], specifically an error.
///
/// An example which can cause this is an improper API response. When implementing, users are required
/// to use the [async_trait](async_trait) macro on top of their implementation.
///
/// Then for notifying the "Scheduler Side" about the results, they do it via [`TriggerNotifier::notify`]
/// method and supply the new future time. For more context look below in the example.
///
/// # Required Subtrait(s)
/// On its own [`TaskTrigger`] does not require any significant traits, it does however need ``'static``
/// lifetime and ``Send + Sync`` auto traits.
///
/// # Implementation(s)
/// While [`TaskTrigger`] by itself has no direct implementations, there are indirect implementations
/// which utilize [`TaskSchedule`](schedule::TaskSchedule).
///
/// # Object Safety / Dynamic Dispatching
/// [`TaskTrigger`] **IS** object safe / dynamic dispatchable without any restrictions.
///
///
/// # Blanket Implementation(s)
/// Any [`TaskSchedule`](schedule::TaskSchedule) automatically implements [`TaskTrigger`].
///
/// It wraps the sync nature of [`TaskSchedule`](schedule::TaskSchedule) to the async world of [`TaskTrigger`], managing the
/// trigger notifier and executing the [`TaskSchedule`](schedule::TaskSchedule).
///
/// # Example(s)
/// ```
/// use std::time::{SystemTime, Duration};
/// use std::error::Error;
/// use chronographer::task::{TaskTrigger, TriggerNotifier};
/// use tokio::time::sleep;
/// use async_trait::async_trait;
///
/// struct DeferredEveryFiveSeconds;
///
/// #[async_trait]
/// impl TaskTrigger for DeferredEveryFiveSeconds {
///     async fn trigger(
///         &self,
///         now: SystemTime,
///         notifier: TriggerNotifier,
///     ) -> Result<(), Box<dyn Error + Send + Sync>> {
///         // Offloads our logic to not block initialization
///         notifier.notify_with(move || async move {
///             sleep(Duration::from_secs(2)).await;
///             Ok(now + Duration::from_secs(5))
///         });
///
///         Ok(())
///     }
/// }
/// ```
///
/// # See Also
/// - [`TriggerNotifier`] - The channel used to notify the "Scheduler Side" when the calculated time is ready.
/// - [`TaskScheduleImmediate`] - For scheduling Tasks to immediately execute.
/// - [`TaskScheduleInterval`] - For scheduling Tasks per interval basis.
/// - [`TaskScheduleCron`] - For scheduling Tasks via a CRON expression (Quartz-style).
/// - [`TaskScheduleCalendar`] - For scheduling Tasks via a human-readable configurable calendar object.
/// - [`Tasks`](Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
/// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
#[async_trait]
pub trait TaskTrigger: 'static + Send + Sync {
    async fn init(&self, now: SystemTime) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// The only required method of [`TaskTrigger`].
    ///
    /// # Semantics
    /// Its job is to calculate the next future time given a current time and optionally
    /// some outside state influencing those calculations.
    ///
    /// These calculations may be deferred and non-immediate which allows flexibility for interacting
    /// with I/O, network-based APIs or anything in-between.
    ///
    /// For this reason, [`TaskTrigger`] returns its results via [`TaskTriggerNotifier`] which is explained
    /// in the ``Argument(s)`` header below.
    ///
    /// When calculations are immediate and more mathematical / computational, it is best to use
    /// [TaskSchedule](schedule::TaskSchedule) and its [`TaskSchedule::schedule`](schedule::TaskSchedule::schedule).
    ///
    /// # Arguments
    /// There are two arguments at play, the first is the "now" argument which
    /// utilizes [`SystemTime`] provided by Rust.
    ///
    /// > **Important Note:** The value for the "now" argument is **NOT** the same as using [`SystemTime::now`],
    /// the value is defined by which [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) is used
    ///
    /// The second argument is a [`TriggerNotifier`] which is the main channel in which the [`TaskTrigger`]
    /// sends its results back to "Scheduler Side".
    ///
    /// > **Important Note #2:** In almost all cases for notifying, it is best to use [`TriggerNotifier::notify_with`]
    /// as to not block
    async fn trigger(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}
