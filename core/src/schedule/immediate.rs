use crate::schedule::TaskSchedule;
use chrono::{DateTime, Local};
use std::sync::Arc;

#[allow(unused_imports)]
use crate::task::Task;

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
