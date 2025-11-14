use crate::persistence::PersistenceObject;
use crate::schedule::TaskSchedule;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

/// [`TaskScheduleCron`] is an implementation of the [`TaskSchedule`] trait that executes [`Task`]
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
/// Apart from implementing [`TaskSchedule`], [`TaskScheduleCron`] also implements:
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
/// use chronographer_core::schedule::TaskScheduleCron;
///
/// let schedule = TaskScheduleCron::new("0 12 * * *".to_owned());
///
/// // Run every 5 minutes
/// let schedule = TaskScheduleCron::new("*/5 * * * *".to_owned());
/// ```
///
/// # See also
/// - [`Task`]
/// - [`ScheduleCalendar`]
/// - [`TaskSchedule`]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(cron_parser::parse(&self.0, time).unwrap())
    }
}

#[async_trait]
impl PersistenceObject for TaskScheduleCron {
    const PERSISTENCE_ID: &'static str =
        "chronographer::TaskScheduleCron#ede44f5f-a3bc-464c-9284-f3c666470cc7";
}
