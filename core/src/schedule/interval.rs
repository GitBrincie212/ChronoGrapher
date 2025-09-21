use crate::schedule::TaskSchedule;
use chrono::{DateTime, Local, TimeDelta};
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;

/// [`TaskScheduleInterval`] is a straightforward implementation of the [`TaskSchedule`] trait
/// that executes tasks at a fixed interval.
///
/// The interval is defined using either a [`TimeDelta`] or a [`Duration`], making it
/// flexible for different time representations. This makes it well-suited for recurring
/// jobs such as periodic cleanup tasks, heartbeat signals, polling operations... etc.
///
/// # Construction
///
/// - Use [`TaskScheduleInterval::new`] to create an interval schedule from a [`TimeDelta`].
/// - Use [`TaskScheduleInterval::duration`] when constructing from a [`Duration`].
///
/// # Examples
///
/// ```ignore
/// use std::time::Duration;
/// use chronographer_core::schedule::TaskScheduleInterval;
///
/// // Run every 5 seconds
/// let schedule = TaskScheduleInterval::duration(Duration::from_secs(5));
/// ```
///
/// # See also
/// - [`TaskSchedule`] — the trait implemented by this type
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy, Default)]
pub struct TaskScheduleInterval(pub(crate) TimeDelta);

impl TaskScheduleInterval {
    pub fn new(interval: TimeDelta) -> Self {
        Self(interval)
    }

    pub fn duration(interval: Duration) -> Self {
        Self(TimeDelta::from_std(interval).unwrap())
    }

    pub fn from_secs(interval: u32) -> Self {
        Self(TimeDelta::seconds(interval as i64))
    }

    pub fn from_secs_f64(interval: f64) -> Self {
        Self(TimeDelta::from_std(Duration::from_secs_f64(interval)).unwrap())
    }
}

impl TaskSchedule for TaskScheduleInterval {
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        Ok(time.add(self.0))
    }
}

macro_rules! integer_from_impl {
    ($val: ty) => {
        impl From<$val> for TaskScheduleInterval {
            fn from(value: $val) -> Self {
                TaskScheduleInterval(TimeDelta::seconds(value as i64))
            }
        }
    };
}

integer_from_impl!(u8);
integer_from_impl!(u16);
integer_from_impl!(u32);

impl From<f64> for TaskScheduleInterval {
    fn from(value: f64) -> Self {
        TaskScheduleInterval::from_secs_f64(value)
    }
}

impl From<f32> for TaskScheduleInterval {
    fn from(value: f32) -> Self {
        TaskScheduleInterval::from_secs_f64(value as f64)
    }
}
