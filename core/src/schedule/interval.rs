use crate::persistent_object::PersistentObject;
use crate::schedule::TaskSchedule;
use crate::serialized_component::SerializedComponent;
use crate::task::TaskError;
use async_trait::async_trait;
use chrono::{DateTime, Local, TimeDelta};
use serde_json::json;
use std::fmt::Debug;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use crate::utils::PersistenceUtils;

#[allow(unused_imports)]
use crate::task::Task;

/// [`TaskScheduleInterval`] is a straightforward implementation of the [`TaskSchedule`] trait
/// that executes [`Task`] instances at a fixed interval. The interval is defined using either a [`TimeDelta`] or
/// a [`Duration`], making it flexible for different time representations. This makes it well-suited
/// for recurring jobs such as periodic cleanup tasks, heartbeat signals, polling operations... etc.
///
/// # Constructor(s)
/// When one wants to create a new [`TaskScheduleInterval`] instance, they can use a variety
/// of constructors, those being:
/// - [`TaskScheduleInterval::new`] Creates a [`TaskScheduleInterval`] with a [`TimeDelta`]
/// - [`TaskScheduleInterval::duration`] Creates a [`TaskScheduleInterval`] with a [`Duration`]
/// - [`TaskScheduleInterval::from_secs`] Creates a [`TaskScheduleInterval`] with an
///   interval number of seconds
/// - [`TaskScheduleInterval::from_secs_f64`] Similar to [`TaskScheduleInterval::from_secs`] but for floating-point
///   numbers for seconds
///
/// One can also construct via ``From`` trait implementations
///
/// # Examples
/// ```ignore
/// use std::time::Duration;
/// use chronographer_core::schedule::TaskScheduleInterval;
///
/// // Run every 5 seconds
/// let schedule = TaskScheduleInterval::duration(Duration::from_secs(5));
/// ```
///
/// # Trait Implementation(s)
/// [`TaskScheduleInterval`] implements obviously the [`TaskSchedule`] trait but also a variety
/// of other traits, those being:
/// - [`Debug`]
/// - [`Clone`]
/// - [`Copy`]
/// - [`Eq`]
/// - [`PartialEq`]
/// - [`PartialOrd`]
/// - [`Ord`]
///
/// In addition, it implements ``From`` trait for various integers and float numbers, those being:
/// - ``u8``
/// - ``u16``
/// - ``u32``
/// - ``f32``
/// - ``f64``
///
/// # See also
/// - [`Task`]
/// - [`TaskSchedule`]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
pub struct TaskScheduleInterval(pub(crate) TimeDelta);

impl TaskScheduleInterval {
    /// Constructs / Creates a new [`TaskScheduleInterval`] instance. There
    /// are various other constructors, suited for other types such as
    /// - [`TaskScheduleInterval::duration`] for [`Duration`]
    /// - [`TaskScheduleInterval::from_secs`] for seconds represented as u32
    /// - [`TaskScheduleInterval::from_secs_f64`] for seconds represented as f64
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being a chrono [`TimeDelta`] interval
    /// as ``interval``
    ///
    /// # Returns
    /// The newly created instance [`TaskScheduleInterval`] with an interval being ``interval``
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`]
    /// - [`TaskScheduleInterval::duration`]
    /// - [`TaskScheduleInterval::from_secs`]
    /// - [`TaskScheduleInterval::from_secs_f64`]
    pub fn new(interval: TimeDelta) -> Self {
        Self(interval)
    }

    /// Constructs / Creates a new [`TaskScheduleInterval`] instance. There
    /// are various other constructors, suited for other types such as
    /// - [`TaskScheduleInterval::new`] for chrono [`TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs`] for seconds represented as u32
    /// - [`TaskScheduleInterval::from_secs_f64`] for seconds represented as f64
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being a [`Duration`] interval
    /// as ``interval``
    ///
    /// # Returns
    /// The newly created instance [`TaskScheduleInterval`] with an interval being ``interval``
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`]
    /// - [`TaskScheduleInterval::new`]
    /// - [`TaskScheduleInterval::from_secs`]
    /// - [`TaskScheduleInterval::from_secs_f64`]
    pub fn duration(interval: Duration) -> Self {
        Self(TimeDelta::from_std(interval).unwrap())
    }

    /// Constructs / Creates a new [`TaskScheduleInterval`] instance. There
    /// are various other constructors, suited for other types such as
    /// - [`TaskScheduleInterval::duration`] for [`Duration`]
    /// - [`TaskScheduleInterval::new`] for chrono [`TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs_f64`] for seconds represented as f64
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``u32`` number representing the number
    /// of seconds of an interval as ``interval``
    ///
    /// # Returns
    /// The newly created instance [`TaskScheduleInterval`] with an interval being ``interval``
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`]
    /// - [`TaskScheduleInterval::duration`]
    /// - [`TaskScheduleInterval::new`]
    /// - [`TaskScheduleInterval::from_secs_f64`]
    pub fn from_secs(interval: u32) -> Self {
        Self(TimeDelta::seconds(interval as i64))
    }

    /// Constructs / Creates a new [`TaskScheduleInterval`] instance. There
    /// are various other constructors, suited for other types such as
    /// - [`TaskScheduleInterval::duration`] for [`Duration`]
    /// - [`TaskScheduleInterval::from_secs`] for seconds represented as u32
    /// - [`TaskScheduleInterval::new`] for chrono [`TimeDelta`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``f64`` number representing the number
    /// of seconds of an interval as ``interval``
    ///
    /// # Returns
    /// The newly created instance [`TaskScheduleInterval`] with an interval being ``interval``
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`]
    /// - [`TaskScheduleInterval::duration`]
    /// - [`TaskScheduleInterval::from_secs`]
    /// - [`TaskScheduleInterval::new`]
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

#[async_trait]
impl PersistentObject for TaskScheduleInterval {
    fn persistence_id() -> &'static str {
        "TaskScheduleInterval$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let secs = PersistenceUtils::serialize_field(self.0.as_seconds_f64())?;
        Ok(SerializedComponent::new::<Self>(json!({
            "interval_seconds": secs
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<TaskScheduleInterval, TaskError> {
        let mut map = PersistenceUtils::transform_serialized_to_map(component)?;

        let interval = PersistenceUtils::deserialize_atomic::<f64>(
            &mut map,
            "seconds",
            "Cannot deserialize the interval_seconds field"
        )?;

        Ok(TaskScheduleInterval::from_secs_f64(interval))
    }
}
