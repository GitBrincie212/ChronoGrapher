use crate::deserialization_err;
use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::schedule::TaskSchedule;
use crate::serialized_component::SerializedComponent;
use crate::task::TaskError;
use crate::{acquire_mut_ir_map, deserialize_field, to_json};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde_json::json;
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
/// Apart from implementing [`TaskSchedule`], [`TaskScheduleCron`] also implements the [`Debug`] trait,
/// the [`Clone`] trait, the [`Eq`] trait and subsequently the [`PartialEq`] trait
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
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(cron_parser::parse(&self.0, time).unwrap())
    }
}

#[async_trait]
impl PersistentObject for TaskScheduleCron {
    fn persistence_id() -> &'static str {
        "TaskScheduleCron$chronographer_core"
    }

    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        let cron = to_json!(self.0.as_str());
        Ok(SerializedComponent::new::<Self>(json!({
            "cron_expression": cron
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<TaskScheduleCron, TaskError> {
        let mut map = acquire_mut_ir_map!(TaskScheduleCron, component);

        deserialize_field!(
            map,
            serialized_cron,
            "cron_expression",
            TaskScheduleCron,
            "Cannot deserialize the cron_expression field"
        );

        let cron = serialized_cron
            .as_str()
            .ok_or_else(|| {
                deserialization_err!(
                    map,
                    TaskScheduleCron,
                    "Cannot deserialize the cron_expression field"
                )
            })?
            .to_string();

        Ok(TaskScheduleCron::new(cron))
    }
}
