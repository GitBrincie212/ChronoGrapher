use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;
use async_trait::async_trait;
use std::time::SystemTime;

#[derive(Default)]
pub struct ProgressiveClock;

impl Clone for ProgressiveClock {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for ProgressiveClock {}

#[async_trait]
impl<C: SchedulerConfig> SchedulerClock<C> for ProgressiveClock {
    async fn now(&self) -> SystemTime {
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
}
