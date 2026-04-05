use crate::scheduler::clock::SchedulerClock;
#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;
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

impl SchedulerClock for ProgressiveClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn idle_to(&self, to: SystemTime) -> impl Future<Output = ()> + Send {
        let now = SystemTime::now();
        let duration = to.duration_since(now).unwrap_or(Duration::ZERO);

        tokio::time::sleep(duration)
    }

    fn tick(&self) -> impl Future<Output = ()> + Send {
        self.0.notified()
    }
}
