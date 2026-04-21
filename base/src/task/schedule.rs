//! This module contains various implementations of scheduling primitives via [`TaskSchedule`](crate::task::TaskSchedule).
//!
//! When it comes to most use cases, the built-in scheduling primitives are most used. However, depending
//! on your needs, you may implement the [`TaskSchedule`](crate::task::TaskSchedule) trait for a custom schedule.
//!
//! # Exports
//! - [`TaskScheduleImmediate`] - A primitive which schedules to execute immediately.
//! - [`TaskScheduleInterval`] - A primitive which schedules per-interval basis.
//! - [`TaskScheduleCron`] - A primitive which schedules based on a CRON expression.
//! - [`CronField`] - A field used internally for [`TaskScheduleCron`]
//! - [`TaskScheduleCalendar`] - A primitive which schedules via a human-readable calendar object.
//! - [`TaskCalendarField`] - A field of [`TaskScheduleCalendar`] which allows complex scheduling.
//!
//! # Example(s)
//! TODO: Expand upon the Example(s) once you are finished with documenting the other primitives
//!
//! Implementing your own custom schedule? Best refer to [`TaskSchedule`](crate::task::TaskSchedule) documentation
//!
//! # See Also
//! - [`TaskScheduleImmediate`] - A primitive which schedules to execute immediately.
//! - [`TaskScheduleInterval`] - A primitive which schedules per-interval basis.
//! - [`TaskScheduleCron`] - A primitive which schedules based on a CRON expression.
//! - [`CronField`] - A field used internally for [`TaskScheduleCron`]
//! - [`TaskScheduleCalendar`] - A primitive which schedules via a human-readable calendar object.
//! - [`TaskCalendarField`] - A field of [`TaskScheduleCalendar`] which allows complex scheduling.
//! - [`TaskSchedule`](crate::task::TaskSchedule) - The trait for managing scheduling / trigger logic.

mod calendar; // skipcq: RS-D1001
mod cron; // skipcq: RS-D1001
mod immediate;
mod interval; // skipcq: RS-D1001

pub mod cron_lexer; // skipcq: RS-D1001
pub mod cron_parser; // skipcq: RS-D1001

use std::error::Error;
use std::time::SystemTime;
use async_trait::async_trait;

pub use calendar::*;
pub use cron::*;
pub use immediate::*;
pub use interval::*;

/// [`TaskSchedule`] is the main mechanism in which [`Tasks`](crate::task::Task) schedule a future time (based on
/// a current one) to run, this time is handed to the "[`Scheduler`](crate::scheduler::Scheduler) Side"
/// for it to organize.
///
/// [`TaskSchedule`] may immediately hand out the future time (in this case, best use [`TaskSchedule`](schedule::TaskSchedule)
/// or notify at any other time the "Scheduler Side" about its future time to schedule to.
///
/// # Semantics
/// There is only one required method for the [`TaskSchedule`], that being [`TaskSchedule::schedule`].
///
/// When implementing, users are required to use the [async_trait](async_trait) macro on top of their
/// implementation, then implement [`TaskSchedule::schedule`].
///
/// # Required Subtrait(s)
/// On its own [`TaskSchedule`] does not require any significant traits, it does however need ``'static``
/// lifetime and ``Send + Sync`` auto traits.
///
/// # Implementation(s)
/// While [`TaskSchedule`] by itself has no direct implementations, there are indirect implementations
/// which utilize [`TaskSchedule`](schedule::TaskSchedule).
///
/// # Object Safety / Dynamic Dispatching
/// [`TaskSchedule`] **IS** object safe / dynamic dispatchable without any restrictions.
///
///
/// # Blanket Implementation(s)
/// Any [`TaskSchedule`](schedule::TaskSchedule) automatically implements [`TaskSchedule`].
///
/// It wraps the sync nature of [`TaskSchedule`](schedule::TaskSchedule) to the async world of [`TaskSchedule`], managing the
/// trigger notifier and executing the [`TaskSchedule`](schedule::TaskSchedule).
///
/// # Example(s)
/// ```
/// use std::time::{SystemTime, Duration};
/// use std::error::Error;
/// use chronographer::task::TaskSchedule;
/// use tokio::time::sleep;
/// use async_trait::async_trait;
///
/// struct DeferredEveryFiveSeconds;
///
/// #[async_trait]
/// impl TaskSchedule for DeferredEveryFiveSeconds {
///     // By default init() returns Ok(()) every time. You can specify your own logic
///     // if needed, by implementing the init(...) method from TaskSchedule
///
///     async fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
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
/// let future_time = instance.schedule(now).await?;
/// let elapsed = instant.elapsed().as_secs_f64();
///
/// // Checks the time the schedule took (the 10ms is for accounting some variability)
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
pub trait TaskSchedule: 'static + Send + Sync {
    /// The only required method of [`TaskSchedule`], it hosts the actual logic of waiting,
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
    /// For a complete example on how to implement this method, it is best to view [`TaskSchedule`].
    ///
    /// # See Also
    /// - [`TaskSchedule`] - The main trait that holds this method
    /// - [TaskSchedule](schedule::TaskSchedule) - An alias from this trait for more immediate mathematical computation.
    /// - [`Tasks`](crate::task::Task) - The main container which the schedule is hosted on.
    /// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
    /// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
    async fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}