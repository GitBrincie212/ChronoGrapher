use crate::task::{TaskTrigger, TriggerNotifier};
use async_trait::async_trait;
use std::error::Error;
use std::time::SystemTime;

pub mod calendar; // skipcq: RS-D1001

pub mod cron; // skipcq: RS-D1001

pub mod immediate; // skipcq: RS-D1001

pub mod interval; // skipcq: RS-D1001

pub trait TaskSchedule: 'static + Send + Sync {
    fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl<T: TaskSchedule> TaskTrigger for T {
    async fn trigger(
        &self,
        now: SystemTime,
        notifier: TriggerNotifier,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let date = self.schedule(now)?;
        notifier.notify(Ok(date)).await;
        Ok(())
    }
}
