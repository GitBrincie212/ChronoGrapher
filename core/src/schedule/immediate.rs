use crate::persistent_object::PersistentObject;
use crate::schedule::TaskSchedule;
use crate::serialized_component::SerializedComponent;
#[allow(unused_imports)]
use crate::task::Task;
use crate::task::TaskError;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde_json::json;
use std::sync::Arc;

/// [`TaskScheduleImmediate`] is an implementation of the [`TaskSchedule`] trait
/// that executes any [`Task`] instance immediately once scheduled / rescheduled
///
/// # Constructor(s)
/// If one wishes to construct a [`TaskScheduleImmediate`], they can simply use
/// rust's struct initialization by just dropping [`TaskScheduleImmediate`] or with
/// [`TaskScheduleImmediate::default`] via [`Default`] trait
///
/// # Trait Implementation(s)
/// Obviously, [`TaskScheduleImmediate`] implements the [`TaskSchedule`] trait but
/// also [`Debug`] (just prints the name, nothing more, nothing less), [`Clone`], [`Copy`]
/// and [`Default`]
///
/// # See also
/// - [`Task`]
/// - [`TaskSchedule`]
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskScheduleImmediate;

impl TaskSchedule for TaskScheduleImmediate {
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(*time)
    }
}

#[async_trait]
impl PersistentObject<TaskScheduleImmediate> for TaskScheduleImmediate {
    fn persistence_id(&self) -> &'static str {
        "TaskScheduleImmediate$chronographer_core"
    }

    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        Ok(SerializedComponent::new(
            self.persistence_id().to_string(),
            json!({}),
        ))
    }

    async fn retrieve(
        _component: SerializedComponent,
    ) -> Result<TaskScheduleImmediate, TaskError> {
        Ok(TaskScheduleImmediate)
    }
}
