use crate::task::{TaskTrigger, TriggerNotifier};
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

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
/// method present in this trait.
///
/// It takes the current time as a ``SystemTime`` (via "now" argument) and computes the next time returning
/// it as a Result which can be either the new future ``SystemTime`` or an error if failed.
///
/// > **Important Note:** The value for the "now" argument is not the same as using [`SystemTime::now`],
/// the value is defined by which [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) is used
///
/// # Required Subtrait(s)
/// On its own [`TaskSchedule`] does not require any significant traits, it does however need ``'static``
/// lifetime and ``Send + Sync`` auto traits.
///
/// # Object Safety / Dynamic Dispatching
/// [`TaskSchedule`] **IS** object safe / dynamic dispatchable without any restrictions.
///
/// # Blanket Implementation(s)
/// As discussed above, any [`TaskSchedule`] automatically implements the more generalized [`TaskTrigger`]
/// system for anything that requires alerting the "Scheduler Side" about time.
///
/// It wraps the sync nature of [`TaskSchedule`] to the async world of [`TaskTrigger`], managing the
/// trigger notifier and executing the [`TaskSchedule`].
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
/// - [`TaskTrigger`] - The main system used for notifying the "Scheduler Side" for scheduling a Task.
/// - [`TriggerNotifier`] - A channel used by the trigger to notify the "Scheduler Side" when the calculated time is ready.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
/// - [`SchedulerClock`](crate::scheduler::clock::SchedulerClock) - The mechanism that supplies the "now" argument with the value
pub trait TaskSchedule: 'static + Send + Sync {
    fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl<T: TaskSchedule> TaskTrigger for T {
    async fn trigger(
        &self,
        now: SystemTime,
        notifier: TriggerNotifier,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let date = self.schedule(now)?;
        notifier.notify(Ok(date)).await;
        Ok(())
    }
}
