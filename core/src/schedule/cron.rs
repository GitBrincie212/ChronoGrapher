use crate::schedule::TaskSchedule;
use chrono::{DateTime, Local};
use std::sync::Arc;

/// [`TaskScheduleCron`] is an implementation of the [`TaskSchedule`] trait that executes tasks
/// according to a cron expression.
///
/// Cron expressions provide a powerful way to define recurring schedules with fine-grained
/// control (e.g., "every minute", "at 2:30 AM every day", "every Monday at 9 AM").
/// The expression is supplied as a string and parsed when running [`TaskScheduleCron::next_after`].
/// The only drawback compared to something like [`ScheduleCalendar`] is the inability to
/// have second and millisecond precision.
///
/// # Construction
///
/// Use [`TaskScheduleCron::new`] to create a schedule from a cron expression string.
///
/// # Examples
///
/// ```ignore
/// // Run at 12:00 (noon) every day
/// use chronographer_core::schedule::TaskScheduleCron;
///
/// let schedule = TaskScheduleCron::new("0 12 * * *".to_owned());
///
/// // Run every 5 minutes
/// let schedule = TaskScheduleCron::new("*/5 * * * *".to_owned());
/// ```
///
/// # See also
/// - [`TaskSchedule`] — the trait implemented by this type
/// - [`ScheduleCalendar`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TaskScheduleCron(pub(crate) String);

impl TaskScheduleCron {
    pub fn new(cron: String) -> Self {
        Self(cron)
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(cron_parser::parse(&self.0, time).unwrap())
    }
}
