use crate::persistence::{PersistenceContext, PersistenceObject};
#[allow(unused_imports)]
use crate::task::Task;
use crate::task::TaskSchedule;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
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
/// Obviously, [`TaskScheduleImmediate`] implements the [`TaskSchedule`] trait but also:
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
/// - [`TaskSchedule`]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TaskScheduleImmediate;

impl TaskSchedule for TaskScheduleImmediate {
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(*time)
    }
}

impl PersistenceObject for TaskScheduleImmediate {
    const PERSISTENCE_ID: &'static str =
        "chronographer::TaskScheduleImmediate#74c56b86-2a45-4d18-abc7-d2da47218a28";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}
