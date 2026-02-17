use crate::errors::StandardCoreErrorsCG;
use crate::task::schedule::TaskSchedule;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt::Debug;
use std::time::SystemTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TaskScheduleCron(String);

impl TaskScheduleCron {
    pub fn new(cron: String) -> Self {
        Self(cron)
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        let dt = DateTime::<Utc>::from(time);
        cron_parser::parse(&self.0, &dt)
            .map_err(|e| {
                Box::new(StandardCoreErrorsCG::CronParserError(e.to_string()))
                    as Box<dyn Error + Send + Sync>
            })
            .map(SystemTime::from)
    }
}
