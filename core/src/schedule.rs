pub mod calendar; // skipcq: RS-D1001

pub mod cron; // skipcq: RS-D1001

pub mod immediate; // skipcq: RS-D1001

pub mod interval; // skipcq: RS-D1001

pub use crate::schedule::calendar::TaskCalendarField;
pub use crate::schedule::calendar::TaskScheduleCalendar;
pub use crate::schedule::cron::TaskScheduleCron;
pub use crate::schedule::immediate::TaskScheduleImmediate;
pub use crate::schedule::interval::TaskScheduleInterval;

use chrono::{DateTime, Local};
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::task::Task;

/// The [`TaskSchedule`] trait is used to calculate the next point of time given a time instance
/// where the task will be scheduled to execute. This system is used closely by the [`Scheduler`]
/// and the [`Task`]
///
/// # Required Method(s)
/// If one wants to implement this trait, they must provide an implementation for the
/// [`TaskSchedule::next_after`] method used to calculate the next available time
///
/// # Trait Implementation(s)
/// some of the noteworthy trait implementation of this trait include:
/// - [`TaskScheduleInterval`] executes a task on an interval basis
/// - [`TaskScheduleCalendar`] executes a task based on the provided cron expression as a string
/// - [`TaskScheduleCron`] defines a human-friendly schedule on when the task runs, it provides fine-grain
///   control on each individual field via [`TaskCalendarField`], it can be at an exact date, an interval basis... etc.
///   It is a good alternative to cron, as it provides second and millisecond accuracy plus being more human-friendly
///
/// This trait is also implemented for any type implementing ``Deref`` where the target is ``T`` which
/// itself is an implementation of the [`TaskSchedule`] trait, making it relatively easy to store both
/// owned and non-owned values
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`Task`] struct
///
/// # See Also
/// - [`Scheduler`]
/// - [`TaskScheduleInterval`]
/// - [`TaskScheduleCalendar`]
/// - [`TaskScheduleCron`]
/// - [`Task`]
pub trait TaskSchedule: Send + Sync {
    /// Calculates the next point in time to schedule the [`Task`] via a specific point in time. This
    /// method is called automatically by the [`Scheduler`] in either registration or rescheduling
    ///
    /// # Arguments
    /// It accepts a ``time`` reference which is a local time used as a basis for calculating the
    /// future time to execute at
    ///
    /// # Returns
    /// A ``Result<DateTime<Local>, Arc<dyn Error>>`` which when successful, returns the calculated
    /// local time, otherwise it returns an error wrapped in an Arc
    ///
    /// # See Also
    /// - [`Scheduler`]
    /// - [`TaskSchedule`]
    /// - [`Task`]
    fn next_after(&self, time: &DateTime<Local>) -> Result<DateTime<Local>, Arc<dyn Error>>;
}

impl<T> TaskSchedule for T
where
    T: Deref + Send + Sync,
    T::Target: TaskSchedule,
{
    fn next_after(&self, time: &DateTime<Local>) -> Result<DateTime<Local>, Arc<dyn Error>> {
        self.deref().next_after(time)
    }
}
