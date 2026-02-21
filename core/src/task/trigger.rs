pub mod schedule; // skipcq: RS-D1001

use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::SchedulePayload;
#[allow(unused_imports)]
use crate::task::Task;
pub use crate::task::trigger::schedule::calendar::TaskCalendarField;
pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
pub use crate::task::trigger::schedule::cron::TaskScheduleCron;
pub use crate::task::trigger::schedule::immediate::TaskScheduleImmediate;
pub use crate::task::trigger::schedule::interval::TaskScheduleInterval;
use async_trait::async_trait;
use std::any::Any;
use std::error::Error;
use std::time::SystemTime;

pub struct TriggerNotifier {
    id: Box<dyn Any + Send + Sync>,
    notify: tokio::sync::mpsc::Sender<SchedulePayload>,
}

impl TriggerNotifier {
    pub fn new<C: SchedulerConfig>(
        id: <C as SchedulerConfig>::TaskIdentifier,
        notify: tokio::sync::mpsc::Sender<SchedulePayload>,
    ) -> Self {
        Self {
            id: Box::new(id),
            notify,
        }
    }

    pub async fn notify(self, time: Result<SystemTime, Box<dyn Error + Send + Sync>>) {
        self.notify
            .send((self.id, time))
            .await
            .expect("Failed to send notification via TaskTrigger, could not receive from the SchedulerTaskStore");
    }
}

#[async_trait]
pub trait TaskTrigger: 'static + Send + Sync {
    async fn trigger(
        &self,
        now: SystemTime,
        notifier: TriggerNotifier,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
