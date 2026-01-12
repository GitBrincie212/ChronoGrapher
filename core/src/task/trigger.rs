pub mod schedule; // skipcq: RS-D1001

use std::any::Any;
pub use crate::task::trigger::schedule::calendar::TaskCalendarField;
pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
pub use crate::task::trigger::schedule::cron::TaskScheduleCron;
pub use crate::task::trigger::schedule::immediate::TaskScheduleImmediate;
pub use crate::task::trigger::schedule::interval::TaskScheduleInterval;

use crate::prelude::TaskError;
#[allow(unused_imports)]
use crate::task::Task;
use std::time::SystemTime;
use async_trait::async_trait;
use crate::scheduler::SchedulerConfig;

pub struct TriggerNotifier {
    id: Box<dyn Any + Send + Sync>,
    notify: tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Result<SystemTime, TaskError>)>
}

impl TriggerNotifier {
    pub fn new<C: SchedulerConfig>(
        id: <C as SchedulerConfig>::TaskIdentifier, 
        notify: tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Result<SystemTime, TaskError>)>) -> Self
    {
        Self { id: Box::new(id), notify }
    }

    pub async fn notify(self, time: Result<SystemTime, TaskError>) {
        self.notify
            .send((self.id, time))
            .await
            .expect("Failed to send notification via TaskTrigger, could not receive from the SchedulerTaskStore");
    }
}

#[async_trait]
pub trait TaskTrigger: 'static + Send + Sync {
    async fn trigger(&self, now: SystemTime, notifier: TriggerNotifier) -> Result<(), TaskError>;
}