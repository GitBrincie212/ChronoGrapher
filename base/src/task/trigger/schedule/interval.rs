///! A standalone module containing only the [`TaskScheduleInterval`] scheduling primitive

use crate::task::schedule::TaskSchedule;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Add;
use std::time::{Duration, SystemTime};

/// [`TaskScheduleInterval`] is a [`TaskSchedule`] used to execute a [Task](crate::task::Task) in an
/// interval basis (based on the current time).
///
/// # Scheduling Semantics
/// [`TaskScheduleInterval`] contains an interval which it uses to calculate the new future time
/// by taking the current time plus the interval.
///
/// # Schedule Errors
/// Due to its simplicity, [`TaskScheduleInterval`] will **NEVER** return any kind of error.
///
/// # Constructor(s)
/// There are various ways one can construct a [`TaskScheduleInterval`] instance:
/// - [`TaskScheduleInterval::duration`] - Constructs it via a [`Duration`] object
/// - [`TaskScheduleInterval::from_secs`] - Constructs it via a ``u64`` number (as seconds)
/// - [`TaskScheduleInterval::from_secs_f64`] - Constructs it via a float number (as seconds), **may panic**.
/// - [`TaskScheduleInterval::from`] - Supports unsigned integers up to ``u64`` and even ``f32`` or ``f64``,
/// (for float numbers it **may panic**).
/// - [`TaskScheduleInterval::timedelta`] - Gated behind the ``chrono`` feature, but supports the construction
/// via ``TimeDelta``
///
/// # Trait Implementation(s)
/// Apart from [`TaskScheduleInterval`] implementing the [`TaskSchedule`] trait, it implements as well:
/// - [`Debug`]
/// - [`Clone`]
/// - [`Copy`]
///
/// # Example(s)
/// ```rust
/// use chronographer::task::{TaskScheduleInterval, TaskTrigger};
/// use std::time::{SystemTime, Duration};
/// # use std::error::Error;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let instance = TaskScheduleInterval::from_secs(1);
/// let now = SystemTime::now();
///
/// // Both schedule and trigger methods return the same result
/// let new_time = instance.trigger(now).await?;
///
/// // new_time == now + 1 second.
/// # let a = new_time;
/// # let b = now + Duration::from_secs(1);
/// # assert!(
/// #         a.duration_since(b).unwrap_or(b.duration_since(a).unwrap()).as_secs_f64() <= 0.01,
/// #         "{a:?} isn't approximately equal to {b:?} with tolerance 0.01"
/// #     );
/// # Ok(())
/// # }
/// ```
/// In the example above, we create an instance of [`TaskScheduleInterval`], compute its time via ``trigger``
/// method, the result is ``new_time`` is approximately equal to ``now + 1 second`` (within a small tolerance).
///
/// # Feature Gated?
/// The constructor [`TaskScheduleInterval::timedelta`] is gated behind the ``chrono`` feature. It is
/// meant to support construction of chrono's ``TimeDelta``, enable the feature to use it.
///
/// # See Also
/// - [`TaskSchedule`] - The direct implementor of this trait.
/// - [TaskTrigger](crate::task::TaskTrigger) - The general trait which is implemented under the hood.
/// - [`Task`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
#[derive(Debug, Clone, Copy)]
pub struct TaskScheduleInterval(pub(crate) Duration);

impl TaskScheduleInterval {
    #[cfg(feature = "chrono")]
    pub fn timedelta(
        interval: chrono::TimeDelta,
    ) -> Result<Self, crate::errors::StandardCoreErrorsCG> {
        Ok(Self(interval.to_std().map_err(|_| {
            crate::errors::StandardCoreErrorsCG::IntervalTimedeltaOutOfRange
        })?))
    }

    pub fn duration(interval: Duration) -> Self {
        Self(interval)
    }

    pub fn from_secs(interval: u64) -> Self {
        Self(Duration::from_secs(interval))
    }

    pub fn from_secs_f64(interval: f64) -> Self {
        Self(Duration::from_secs_f64(interval))
    }
}

impl TaskSchedule for TaskScheduleInterval {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        Ok(time.add(self.0))
    }
}

macro_rules! integer_from_impl {
    ($val: ty) => {
        impl From<$val> for TaskScheduleInterval {
            fn from(value: $val) -> Self {
                TaskScheduleInterval(Duration::from_secs(value as u64))
            }
        }
    };
}

integer_from_impl!(u8);
integer_from_impl!(u16);
integer_from_impl!(u32);
integer_from_impl!(u64);

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
