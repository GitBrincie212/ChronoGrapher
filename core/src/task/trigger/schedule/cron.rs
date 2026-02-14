use crate::errors::StandardCoreErrorsCG;
use crate::task::DynArcError;
use crate::task::schedule::TaskSchedule;
use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TaskScheduleCron(String);

impl TaskScheduleCron {
    pub fn new(cron: String) -> Self {
        Self(cron)
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, DynArcError> {
        let dt = DateTime::<Utc>::from(time);
        cron_parser::parse(&self.0, &dt)
            .map_err(|e| Arc::new(StandardCoreErrorsCG::CronParserError(e.to_string())) as DynArcError)
            .map(|x| SystemTime::from(x))
    }
}
