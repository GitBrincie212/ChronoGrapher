use crate::prelude::TaskError;
#[allow(unused_imports)]
use crate::task::Task;
use crate::task::schedule::TaskSchedule;
use std::time::SystemTime;

/// [`TaskScheduleImmediate`] is an implementation of the [`TaskTrigger`] trait
/// that executes any [`Task`] instance immediately once scheduled / rescheduled
///
/// # Constructor(s)
/// If one wishes to construct a [`TaskScheduleImmediate`], they can simply use
/// rust's struct initialization by just dropping [`TaskScheduleImmediate`] or with
/// [`TaskScheduleImmediate::default`] via [`Default`] trait
///
/// # Trait Implementation(s)
/// Obviously, [`TaskScheduleImmediate`] implements the [`TaskTrigger`] trait but also:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See also
/// - [`Task`]
/// - [`TaskTrigger`]
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskScheduleImmediate;

impl TaskSchedule for TaskScheduleImmediate {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, TaskError> {
        Ok(time)
    }
}
