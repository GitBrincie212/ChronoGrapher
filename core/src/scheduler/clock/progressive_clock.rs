use crate::scheduler::clock::SchedulerClock;
#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Notify;

pub struct ProgressiveClock(Arc<Notify>);

impl Default for ProgressiveClock {
    fn default() -> Self {
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1));

            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                interval.tick().await;
                notify_clone.notify_waiters();
            }
        });

        Self(notify)
    }
}

#[async_trait]
impl SchedulerClock for ProgressiveClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    async fn idle_to(&self, to: SystemTime) {
        let now = SystemTime::now();
        let duration = match to.duration_since(now) {
            Ok(duration) => duration,
            Err(_) => {
                return;
            }
        };

        tokio::time::sleep(duration).await;
    }

    async fn tick(&self) {
        self.0.notified().await
    }
}
