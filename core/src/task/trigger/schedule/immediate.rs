use crate::task::schedule::TaskSchedule;
use std::error::Error;
use std::time::SystemTime;

/// [`TaskScheduleImmediate`] is a [`TaskSchedule`] used to immediately execute a [Task](crate::task::Task) up front,
/// without calculating a future time.
///
/// # Scheduling Semantics
/// Since [`TaskScheduleImmediate`] is meant to be immediate, it always returns the current time
/// (acting as an identity function), once the "[Scheduler](crate::scheduler::Scheduler) Side" receives it,
/// it immediately executes said [Task](crate::task::Task).
///
/// # Schedule Errors
/// As a result from above, [`TaskScheduleImmediate`] will **NEVER** return any kind of error.
///
/// # Constructor(s)
/// Since [`TaskScheduleImmediate`] doesn't host any state, it can be constructed via using it as a value
/// or alternatively via [`Default`] trait using the [`TaskScheduleImmediate::default`] constructor.
///
/// # Trait Implementation(s)
/// Apart from [`TaskScheduleImmediate`] implementing the [`TaskSchedule`] trait, it implements as well:
/// - [`Debug`]
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
///
/// # Example(s)
/// ```rust
/// use chronographer::task::{TaskScheduleImmediate, TaskTrigger};
/// use std::time::SystemTime;
/// # use std::error::Error;
/// # use chronographer::task::trigger::schedule::TaskSchedule;
///
/// #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
///     let immediate_schedule = TaskScheduleImmediate;
///     let now = SystemTime::now();
///
///     // Both schedule and trigger methods return the same result
///     let future_time = immediate_schedule.trigger(now).await?;
///
///     assert_eq!(future_time, now);
///     # Ok(())
/// # }
/// ```
///
/// # See Also
/// - [`TaskSchedule`] - The direct implementor of this trait.
/// - [TaskTrigger](crate::task::TaskTrigger) - The general trait which is implemented under the hood.
/// - [`Task`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskScheduleImmediate;

impl TaskSchedule for TaskScheduleImmediate {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        Ok(time)
    }
}
