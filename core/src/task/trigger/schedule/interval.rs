use crate::errors::StandardCoreErrorsCG;
#[allow(unused_imports)]
use crate::task::Task;
use crate::task::schedule::TaskSchedule;
use chrono::TimeDelta;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
pub struct TaskScheduleInterval(pub(crate) Duration);

impl TaskScheduleInterval {
    pub fn timedelta(interval: TimeDelta) -> Result<Self, StandardCoreErrorsCG> {
        Ok(Self(interval.to_std().map_err(|_| {
            StandardCoreErrorsCG::IntervalTimedeltaOutOfRange
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
