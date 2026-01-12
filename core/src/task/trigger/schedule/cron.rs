use crate::errors::ChronographerErrors;
use crate::task::TaskError;
use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;
use crate::task::schedule::TaskSchedule;

/// [`TaskScheduleCron`] is an implementation of the [`TaskTrigger`] trait that executes [`Task`]
/// instances, according to a cron expression. Learn more about cron expression in
/// [Wikipedia](https://en.wikipedia.org/wiki/Cron)
///
/// # Implementation Detail(s)
/// Under the hood, this uses the crate ``cron_parser`` to calculate the new time to execute
///
/// # Usage Note(s)
/// Cron expressions provide a powerful way to define recurring schedules with fine-grained
/// control (e.g., "every minute", "at 2:30 AM every day", "every Monday at 9 AM").
/// The expression is supplied as a string and parsed when running [`TaskScheduleCron::next_after`].
///
/// The only drawback compared to something like [`ScheduleCalendar`] is the inability to
/// have second and millisecond precision.
///
/// # Construction
/// When constructing [`TaskScheduleCron`], the only way to do so is via [`TaskScheduleCron::new`]
/// which requires a cron expression as a string
///
/// # Trait Implementation(s)
/// Apart from implementing [`TaskTrigger`], [`TaskScheduleCron`] also implements:
/// - [`Debug`],
/// - [`Clone`]
/// - [`Eq`]
/// - [`PartialEq`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # Examples
///
/// ```ignore
/// // Run at 12:00 (noon) every day
/// use chronographer_core::trigger::TaskScheduleCron;
///
/// let trigger = TaskScheduleCron::new("0 12 * * *".to_owned());
///
/// // Run every 5 minutes
/// let trigger = TaskScheduleCron::new("*/5 * * * *".to_owned());
/// ```
///
/// # See also
/// - [`Task`]
/// - [`ScheduleCalendar`]
/// - [`TaskTrigger`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TaskScheduleCron(String);

impl TaskScheduleCron {
    /// Constructs / Creates a [`TaskScheduleCron`] from a provided cron expression
    ///
    /// # Argument(s)
    /// This method accepts one argument, this being the cron expression ``cron`` represented
    /// as an owned ``String``
    ///
    /// # Returns
    /// A fully constructed [`TaskScheduleCron`] from a ``cron`` string
    ///
    /// # See Also
    /// - [`TaskScheduleCron`]
    pub fn new(cron: String) -> Self {
        Self(cron)
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, TaskError> {
        let dt = DateTime::<Utc>::from(time);
        cron_parser::parse(&self.0, &dt)
            .map_err(|e| Arc::new(ChronographerErrors::CronParserError(e.to_string())) as TaskError)
            .map(|x| SystemTime::from(x))
    }
}
