pub mod schedule; // skipcq: RS-D1001

pub use crate::task::trigger::schedule::calendar::TaskCalendarField;
pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
pub use crate::task::trigger::schedule::cron::TaskScheduleCron;
pub use crate::task::trigger::schedule::immediate::TaskScheduleImmediate;
pub use crate::task::trigger::schedule::interval::TaskScheduleInterval;
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

/// [`TaskTrigger`] is the main mechanism in which [`Tasks`](crate::task::Task) schedule a future time (based on
/// a current one) to run, this time is handed to the "[`Scheduler`](crate::scheduler::Scheduler) Side"
/// for it to organize.
///
/// [`TaskTrigger`] may immediately hand out the future time (in this case, best use [`TaskSchedule`](schedule::TaskSchedule)
/// or notify at any other time the "Scheduler Side" about its future time to schedule to.
///
/// # Semantics
/// There is only one required method for the [`TaskTrigger`], that being [`TaskTrigger::trigger`].
///
/// When implementing, users are required to use the [async_trait](async_trait) macro on top of their
/// implementation, then implement [`TaskTrigger::trigger`].
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
/// use chronographer::task::TaskTrigger;
/// use tokio::time::sleep;
/// use async_trait::async_trait;
///
/// struct DeferredEveryFiveSeconds;
///
/// #[async_trait]
/// impl TaskTrigger for DeferredEveryFiveSeconds {
///     // By default init() returns Ok(()) every time. You can specify your own logic
///     // if needed, by implementing the init(...) method from TaskTrigger
///
///     async fn trigger(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
///         sleep(Duration::from_secs(2)).await; // Simulated delay
///         Ok(now + Duration::from_secs(5))
///     }
/// }
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let instance = DeferredEveryFiveSeconds;
///
/// let now = SystemTime::now();
/// let instant = tokio::time::Instant::now();
///
/// let future_time = instance.trigger(now).await?;
/// let elapsed = instant.elapsed().as_secs_f64();
///
/// // Checks the time the trigger took (the 10ms is for accounting some variability)
/// assert!((elapsed - 2f64) <= 0.010, "Expected ~2s, got {}s", elapsed);
///
/// // Checks for the returned value if it's actually correct
/// assert_eq!(future_time, now + Duration::from_secs(5));
/// # Ok(())
/// # }
/// ```
///
/// # See Also
/// - [TaskSchedule](schedule::TaskSchedule) - An alias from this trait for more immediate mathematical computation.
/// - [`TaskScheduleImmediate`] - For scheduling Tasks to immediately execute.
/// - [`TaskScheduleInterval`] - For scheduling Tasks per interval basis.
/// - [`TaskScheduleCron`] - For scheduling Tasks via a CRON expression (Quartz-style).
/// - [`TaskScheduleCalendar`] - For scheduling Tasks via a human-readable configurable calendar object.
/// - [`Tasks`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
/// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
#[async_trait]
pub trait TaskTrigger: 'static + Send + Sync {
    /// The only required method of [`TaskTrigger`], it hosts the actual logic of waiting,
    /// monitoring and calculation co-exist to return a new future time based on a current.
    ///
    /// # Semantics
    /// Its job is to calculate the next future time given a current time and optionally
    /// some outside state influencing those calculations.
    ///
    /// These calculations may be deferred and non-immediate which allows flexibility for interacting
    /// with I/O, network-based APIs or anything in-between.
    ///
    /// When calculations are immediate and more mathematical / computational, it is best to use
    /// [TaskSchedule](schedule::TaskSchedule) and its [`TaskSchedule::schedule`](schedule::TaskSchedule::schedule).
    ///
    /// # Arguments
    /// The only argument is the "now" argument which utilizes [`SystemTime`] provided by Rust.
    ///
    /// > **Important Note:** The value for the "now" argument is **NOT** the same as using [`SystemTime::now`],
    /// the value is defined by which [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) is used.
    ///
    /// # Returns
    /// On success the method returns as a result the calculated time, that time may be older than now,
    /// equal to now or an actual future time.
    ///
    /// On the first two cases, it signals the trigger wants to execute immediately, whereas on the
    /// third it wants to specifically execute at the requested future time.
    ///
    /// If the method fails, it returns a boxed error, allowing inspection of what potentially happened
    /// in the triggering stage.
    ///
    /// # Error(s)
    /// Depending on the implementation, different errors may be thrown, there is no standard error
    /// defined in the trait, the semantic implication of the error is it happened during triggering.
    ///
    /// # Example(s)
    /// For a complete example on how to implement this method, it is best to view [`TaskTrigger`].
    ///
    /// # See Also
    /// - [`TaskTrigger`] - The main trait that holds this method
    /// - [TaskSchedule](schedule::TaskSchedule) - An alias from this trait for more immediate mathematical computation.
    /// - [`Tasks`](crate::task::Task) - The main container which the schedule is hosted on.
    /// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
    /// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
    async fn trigger(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}
