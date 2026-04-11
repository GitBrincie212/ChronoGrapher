//! A standalone module containing only the [`TaskScheduleInterval`] scheduling primitive

use std::error::Error;
use std::fmt::Debug;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use async_trait::async_trait;
use crate::errors::StandardCoreErrorsCG;
use crate::task::TaskTrigger;

/// [`TaskScheduleInterval`] is a [`TaskTrigger`] used to execute a [Task](crate::task::Task) in an
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
///   (for float numbers it **may panic**).
/// - [`TaskScheduleInterval::timedelta`] - Gated behind the ``chrono`` feature, but supports the construction
///   via ``TimeDelta``.
///
/// There exists the [every!](chronographer::prelude::every) macro for creating easily [`TaskScheduleInterval`] with a short and
/// readable duration-based syntax, the macro is gated behind the ``macros`` feature and lives in the
/// procedural macros crate.
///
/// # Trait Implementation(s)
/// Apart from [`TaskScheduleInterval`] implementing the [`TaskTrigger`] trait, it implements as well:
/// - [`Debug`]
/// - [`Clone`]
/// - [`Copy`]
///
/// # Example(s)
/// Basic usage of [`TaskScheduleInterval`]:
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
/// Using the [every!](chronographer::prelude::every) macro:
/// ```rust
/// use chronographer::task::{TaskScheduleInterval, TaskTrigger};
/// use std::time::{SystemTime, Duration};
/// # use std::error::Error;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let instance1 = every!(300ms); // every 300ms
/// let instance2 = every!(5.2s); // every 5.2 seconds
/// let instance3 = every!(13m); // every 13 minutes
/// let instance4 = every!(4h); // every 4 hours
/// let instance5 = every!(30d); // every 30 days
/// let instance6 = every!(1d, 2h); // every 1 day and 2 hours
/// let instance7 = every!(1d, 2.5h); // every 1 day and 2.5 hours
/// let instance8 = every!(1d, 2h, 3m, 4s, 5ms); // every 1 day, 2 hours, 3 minutes, 4 seconds and 5 milliseconds
/// let instance9 = every!(1d /* some comment */ 5ms); // every 1 day, 5 milliseconds
///
/// // Commas are optional
/// # Ok(())
/// # }
/// ```
/// In the above example we create multiple [`TaskScheduleInterval`] instances with the [every!](chronographer::prelude::every)
/// macro which is the recommended approach for any duration known at compile-time.
///
/// # Feature Gated?
/// The [every!](chronographer::prelude::every) is gated behind the ``macros`` feature which is enabled
/// by default (but can be disabled to not include any macros).
///
/// # See Also
/// - [`TaskScheduleInterval::timedelta`] - A constructor for [`chrono::TimeDelta`]
/// - [`TaskScheduleInterval::duration`] - A constructor for [`Duration`]
/// - [`TaskScheduleInterval::from_secs`] - A simple constructor for integer second-based intervals.
/// - [`TaskScheduleInterval::from_secs_f64`] - A simple constructor for floating point second-based intervals.
/// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
/// - [`TaskTrigger`] - The direct implementor of this trait.
/// - [`Task`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
#[derive(Debug, Clone, Copy)]
pub struct TaskScheduleInterval(pub(crate) Duration);

impl TaskScheduleInterval {
    #[cfg(feature = "chrono")]
    /// A constructor for [`TaskScheduleInterval`] via a [`chrono::TimeDelta`].
    ///
    /// # Argument(s)
    /// It accepts one argument and that being [`chrono::TimeDelta`] which represents the
    /// interval-basis the [`TaskScheduleInterval`] will use.
    ///
    /// # Returns
    /// A ``Result`` where on success, it contains the newly constructed [`TaskScheduleInterval`] from
    /// the [`chrono::TimeDelta`] argument and on failure an error message (specifically one described below).
    ///
    /// # Error(s)
    /// The method may return an [IntervalTimedeltaOutOfRange](StandardCoreErrorsCG::IntervalTimedeltaOutOfRange)
    /// if the [`chrono::TimeDelta`] maps to a negative duration.
    ///
    /// # Example(s)
    /// ```rust
    /// use chronographer_base::task::TaskScheduleInterval;
    /// use std::time::Duration;
    /// # use chronographer_base::errors::StandardCoreErrorsCG;
    ///
    /// # fn main() -> Result<(), StandardCoreErrorsCG> {
    /// let time1 = chrono::TimeDelta::seconds(42);
    /// let time2 = chrono::TimeDelta::days(-2);
    ///
    /// let interval1 = TaskScheduleInterval::timedelta(time1);
    /// let interval2 = TaskScheduleInterval::timedelta(time2);
    ///
    /// let success: Duration = interval1.unwrap().into();
    /// let err: StandardCoreErrorsCG = interval2.unwrap_err();
    ///
    /// assert_eq!(success, Duration::from_secs(42));
    /// assert_eq!(err, StandardCoreErrorsCG::IntervalTimedeltaOutOfRange);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`] - The main source which the constructor method is part of.
    /// - [`TaskScheduleInterval::duration`] - A similar constructor but for [`Duration`]
    /// - [`TaskScheduleInterval::from_secs`] - A simpler constructor for integer second-based intervals.
    /// - [`TaskScheduleInterval::from_secs_f64`] - A simpler constructor for floating point second-based intervals.
    /// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
    /// - [`chrono::TimeDelta`] - The value used to construct the [`TaskScheduleInterval`] instance.
    /// - [IntervalTimedeltaOutOfRange](StandardCoreErrorsCG::IntervalTimedeltaOutOfRange) -
    /// The error when the argument doesn't map to a positive duration.
    pub fn timedelta(
        interval: chrono::TimeDelta,
    ) -> Result<Self, StandardCoreErrorsCG> {
        Ok(Self(interval.to_std().map_err(|_| {
            StandardCoreErrorsCG::IntervalTimedeltaOutOfRange
        })?))
    }

    /// A constructor for [`TaskScheduleInterval`] via a [`time::Duration`].
    ///
    /// # Argument(s)
    /// It accepts one argument and that being [`time::Duration`] which represents the
    /// interval-basis the [`TaskScheduleInterval`] will use.
    ///
    /// # Returns
    /// The newly constructed [`TaskScheduleInterval`] from the [`time::Duration`] argument.
    ///
    /// # Example(s)
    /// ```rust
    /// use chronographer_base::task::TaskScheduleInterval;
    /// use std::time::Duration;
    /// # use chronographer_base::errors::StandardCoreErrorsCG;
    ///
    /// # fn main() -> Result<(), StandardCoreErrorsCG> {
    /// let dur = time::Duration::seconds(34);
    /// let interval = TaskScheduleInterval::time_duration(dur);
    /// let interval_dur: Duration = interval.into();
    ///
    /// assert_eq!(interval_dur, Duration::from_secs(34));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`] - The main source which the constructor method is part of.
    /// - [`TaskScheduleInterval::timedelta`] - A similar constructor but for [`chrono::TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs`] - A simpler constructor for integer second-based intervals.
    /// - [`TaskScheduleInterval::from_secs_f64`] - A simpler constructor for floating point second-based intervals.
    /// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
    pub fn time_duration(interval: time::Duration) -> Result<Self, StandardCoreErrorsCG> {
        if interval.is_negative() || interval.is_zero()  {
            return Err(StandardCoreErrorsCG::IntervalSecondsOutOfRange)
        }

        Ok(Self(Duration::try_from(interval).unwrap()))
    }

    /// A constructor for [`TaskScheduleInterval`] via a [`Duration`].
    ///
    /// # Argument(s)
    /// It accepts one argument and that being [`Duration`] which represents the
    /// interval-basis the [`TaskScheduleInterval`] will use.
    ///
    /// # Returns
    /// The newly constructed [`TaskScheduleInterval`] from the [`Duration`] argument.
    ///
    /// # Example(s)
    /// ```rust
    /// use chronographer_base::task::TaskScheduleInterval;
    /// use std::time::Duration;
    /// # use chronographer_base::errors::StandardCoreErrorsCG;
    ///
    /// # fn main() -> Result<(), StandardCoreErrorsCG> {
    /// let dur = Duration::from_secs(34);
    /// let interval = TaskScheduleInterval::duration(dur);
    /// let interval_dur: Duration = interval.into();
    ///
    /// assert_eq!(interval_dur, Duration::from_secs(34));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`] - The main source which the constructor method is part of.
    /// - [`TaskScheduleInterval::timedelta`] - A similar constructor but for [`chrono::TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs`] - A simpler constructor for integer second-based intervals.
    /// - [`TaskScheduleInterval::from_secs_f64`] - A simpler constructor for floating point second-based intervals.
    /// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
    pub fn duration(interval: Duration) -> Self {
        Self(interval)
    }

    /// A constructor for [`TaskScheduleInterval`] via an integer ``u64``.
    ///
    /// # Argument(s)
    /// It accepts one argument and that being type of ``u64`` which represents the
    /// interval-basis **(in seconds)** the [`TaskScheduleInterval`] will use.
    ///
    /// For an alternative constructor method which supports decimal-based seconds, it is recommended to look
    /// into the [`TaskScheduleInterval::from_secs_f64`].
    ///
    /// # Returns
    /// The newly constructed [`TaskScheduleInterval`] from the ``u64`` seconds argument.
    ///
    /// # Example(s)
    /// ```rust
    /// use chronographer_base::task::TaskScheduleInterval;
    /// # use chronographer_base::errors::StandardCoreErrorsCG;
    ///
    /// # fn main() -> Result<(), StandardCoreErrorsCG> {
    /// let interval = TaskScheduleInterval::from_secs(12);
    /// let interval_dur: Duration = interval.into();
    ///
    /// assert_eq!(interval_dur, Duration::from_secs(12));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`] - The main source which the constructor method is part of.
    /// - [`TaskScheduleInterval::timedelta`] - A similar constructor but for [`chrono::TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs`] - A simpler constructor for integer second-based intervals.
    /// - [`TaskScheduleInterval::from_secs_f64`] - A simpler constructor for floating point second-based intervals.
    /// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
    pub fn from_secs(interval: u64) -> Self {
        Self(Duration::from_secs(interval))
    }

    /// A constructor for [`TaskScheduleInterval`] via an ``f64``.
    ///
    /// # Argument(s)
    /// It accepts one argument and that being type of ``f64`` which represents the
    /// interval-basis **(in seconds)** the [`TaskScheduleInterval`] will use.
    ///
    /// The number must be positive, finite and a real number, otherwise an error may appear (explained below).
    ///
    /// For an alternative constructor method for integer-based seconds, it is recommended to look
    /// into the [`TaskScheduleInterval::from_secs`].
    ///
    /// # Returns
    /// A result which on success returns the newly constructed [`TaskScheduleInterval`] from the ``f64``
    /// seconds argument. For failure, it returns a [IntervalSecondsOutOfRange](StandardCoreErrorsCG::IntervalSecondsOutOfRange)
    ///
    /// # Error(s)
    /// The method may return an [IntervalSecondsOutOfRange](StandardCoreErrorsCG::IntervalSecondsOutOfRange)
    /// if the ``f64`` number maps to a negative duration.
    ///
    /// # Example(s)
    /// ```rust
    /// use chronographer_base::task::TaskScheduleInterval;
    /// # use chronographer_base::errors::StandardCoreErrorsCG;
    ///
    /// # fn main() -> Result<(), StandardCoreErrorsCG> {
    /// let interval = TaskScheduleInterval::from_secs(12);
    /// let interval_dur: Duration = interval.into();
    ///
    /// assert_eq!(interval_dur, Duration::from_secs(12));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    /// - [`TaskScheduleInterval`] - The main source which the constructor method is part of.
    /// - [`TaskScheduleInterval::timedelta`] - A similar constructor but for [`chrono::TimeDelta`]
    /// - [`TaskScheduleInterval::from_secs`] - A simpler constructor for integer second-based intervals.
    /// - [`TaskScheduleInterval::from_secs_f64`] - A simpler constructor for floating point second-based intervals.
    /// - [every!](chronographer::prelude::every) - A macro with a readable syntax for defining an interval.
    /// - [IntervalSecondsOutOfRange](StandardCoreErrorsCG::IntervalSecondsOutOfRange) - The error
    ///   when the argument doesn't map to a positive duration.
    pub fn from_secs_f64(interval: f64) -> Result<Self, StandardCoreErrorsCG> {
        if interval.is_sign_negative() || !interval.is_finite() {
            return Err(StandardCoreErrorsCG::IntervalSecondsOutOfRange)
        }

        Ok(Self(Duration::from_secs_f64(interval)))
    }
}

impl From<TaskScheduleInterval> for Duration {
    fn from(value: TaskScheduleInterval) -> Self {
        value.0
    }
}

#[async_trait]
impl TaskTrigger for TaskScheduleInterval {
    async fn trigger(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
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

impl TryFrom<f64> for TaskScheduleInterval {
    type Error = StandardCoreErrorsCG;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        TaskScheduleInterval::from_secs_f64(value)
    }
}

impl TryFrom<f32> for TaskScheduleInterval {
    type Error = StandardCoreErrorsCG;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        TaskScheduleInterval::from_secs_f64(value as f64)
    }
}
