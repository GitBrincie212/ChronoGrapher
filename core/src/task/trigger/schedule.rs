use crate::task::TaskTrigger;
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

pub use calendar::{TaskCalendarField, TaskScheduleCalendar};
pub use cron::{TaskScheduleCron, CronField};
pub use immediate::TaskScheduleImmediate;
pub use interval::TaskScheduleInterval;

pub mod calendar; // skipcq: RS-D1001

pub mod cron; // skipcq: RS-D1001

pub mod immediate; // skipcq: RS-D1001

pub mod interval; // skipcq: RS-D1001

mod cron_parser; // skipcq: RS-D1001
mod cron_lexer; // skipcq: RS-D1001

/// [`TaskSchedule`] is a trait for defining a schedule, it acts as an alias for [`TaskTrigger`]
/// where the use of a [`TriggerNotifier`] to alert the "[`Scheduler`](crate::scheduler::Scheduler) Side"
/// at any time is abstracted.
///
/// # Semantics
/// Since it is an alias to a [`TaskTrigger`], it behaves just like it, both [`TaskSchedule`] and
/// [`TaskTrigger`] utilize [`SystemTime`] provided by Rust to supply the next valid time.
///
/// The main difference between [`TaskSchedule`] and [`TaskTrigger`] is the expectation for the former
/// to compute the next valid time immediately and return it (which is why its sync).
///
/// Whereas the latter may announce which time the [`TaskTrigger`] calculated, whenever it wants to
/// (which is why its async). This can be based on anything triggering at an unknown time.
///
/// Unlike [`TaskSchedule`] which has one error, [`TaskTrigger`] has two periods where it can error
/// out, the first is during initialization and second is once the relevant event has occurred.
///
/// # Required Method(s)
/// When implementing [`TaskSchedule`], developers must implement the [schedule](TaskSchedule::schedule)
/// method present in this trait, it is best to read the method's documentation for more details.
///
/// # Required Subtrait(s)
/// On its own [`TaskSchedule`] does not require any significant traits, it does however need ``'static``
/// lifetime and ``Send + Sync`` auto traits.
///
/// # Implementation(s)
/// There are various implementations of [`TaskSchedule`] present in ChronoGrapher, such as:
/// - [`TaskScheduleImmediate`] - For scheduling [`Tasks`](crate::task::Task) to immediately execute.
/// - [`TaskScheduleInterval`] - For scheduling Tasks per interval basis.
/// - [`TaskScheduleCron`] - For scheduling Tasks via a CRON expression (Quartz-style).
/// - [`TaskScheduleCalendar`] For scheduling Tasks via a human-readable configurable calendar object.
///
/// # Object Safety / Dynamic Dispatching
/// [`TaskSchedule`] **IS** object safe / dynamic dispatchable without any restrictions.
///
/// # Example(s)
/// ```
/// use std::time::{SystemTime, Duration};
/// use std::error::Error;
/// use chronographer::task::schedule::TaskSchedule;
///
/// use chronographer::task::TaskTrigger;
///
/// struct EveryFiveSeconds;
///
/// impl TaskSchedule for EveryFiveSeconds {
///     fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
///         Ok(now + Duration::from_secs(5))
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let instance = EveryFiveSeconds;
/// let new_time = instance.schedule(SystemTime::UNIX_EPOCH)?;
/// assert_eq!(new_time, SystemTime::UNIX_EPOCH + Duration::from_secs(5));
///
/// // Can be turned to a TaskTrigger
/// let trigger_instance: &dyn TaskTrigger = &instance;
/// # Ok(())
/// # }
/// ```
///
/// # See Also
/// - [`TaskScheduleImmediate`] - For scheduling Tasks to immediately execute.
/// - [`TaskScheduleInterval`] - For scheduling Tasks per interval basis.
/// - [`TaskScheduleCron`] - For scheduling Tasks via a CRON expression (Quartz-style).
/// - [`TaskScheduleCalendar`] - For scheduling Tasks via a human-readable configurable calendar object.
/// - [`TaskTrigger`] - The main system used for notifying the "Scheduler Side" for scheduling a Task.
/// - [`TriggerNotifier`] - A channel used by the trigger to notify the "Scheduler Side" when the calculated time is ready.
/// - [`Tasks`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
pub trait TaskSchedule: 'static + Send + Sync {
    /// The only required method of [`TaskSchedule`].
    ///
    /// # Semantics
    /// Its job is to calculate the next future time immediately given a current time and optionally
    /// some outside state influencing those calculations.
    ///
    /// These calculations are more mathematical / computation which are immediate and return
    /// deterministically, for deferred computation, refer to [`TaskTrigger`] and its [`TaskTrigger::trigger`].
    ///
    /// # Argument(s)
    /// It takes the current time as a [`SystemTime`] (via "now" argument) and computes the next time returning
    /// it as a Result which can be either the new future [`SystemTime`] or an error if failed.
    ///
    /// > **Important Note:** The value for the "now" argument is **NOT** the same as using [`SystemTime::now`],
    /// the value is defined by which [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) (lives
    /// in the "[`Scheduler`](crate::scheduler::Scheduler) Side") is used.
    ///
    /// # Returns
    /// The method returns on success the "future" [`SystemTime`] (may return the current or past times
    /// for immediate execution) and on failure a boxed error indicating what went wrong.
    ///
    /// # Example(s)
    /// Refer to [`TaskSchedule`] for a complete example of implementing the trait and this method
    /// specifically, as it is the only required one.
    ///
    /// # See Also
    /// - [`TaskSchedule`] - The main trait this method belongs to
    /// - [`TaskTrigger`] - The main system used for notifying the "Scheduler Side" for scheduling a Task.
    /// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
    /// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
    fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl<T: TaskSchedule> TaskTrigger for T {
    async fn init(&self, _now: SystemTime) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn trigger(
        &self,
        now: SystemTime,
    ) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        self.schedule(now)
    }
}
