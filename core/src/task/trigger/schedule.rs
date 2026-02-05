use crate::prelude::DynArcError;
use crate::task::{TaskTrigger, TriggerNotifier};
use async_trait::async_trait;
use std::time::SystemTime;

pub mod calendar; // skipcq: RS-D1001

pub mod cron; // skipcq: RS-D1001

pub mod immediate; // skipcq: RS-D1001

pub mod interval; // skipcq: RS-D1001

pub trait TaskSchedule: 'static + Send + Sync {
    fn schedule(&self, now: SystemTime) -> Result<SystemTime, DynArcError>;
}

#[async_trait]
impl<T: TaskSchedule> TaskTrigger for T {
    async fn trigger(&self, now: SystemTime, notifier: TriggerNotifier) -> Result<(), DynArcError> {
        let date = self.schedule(now).map_err(DynArcError::from)?;
        notifier.notify(Ok(date)).await;
        Ok(())
    }
}
