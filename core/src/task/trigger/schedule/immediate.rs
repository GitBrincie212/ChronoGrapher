use crate::prelude::DynArcError;
#[allow(unused_imports)]
use crate::task::Task;
use crate::task::schedule::TaskSchedule;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Default)]
pub struct TaskScheduleImmediate;

impl TaskSchedule for TaskScheduleImmediate {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, DynArcError> {
        Ok(time)
    }
}
