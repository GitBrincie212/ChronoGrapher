use crate::scheduler::clock::{AdvanceableSchedulerClock, SchedulerClock};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::scheduler::clock::ProgressiveClock;

#[derive(Default)]
pub struct VirtualClock {
    current_time: AtomicU64,
    notify: Notify,
    ticks_buff: AtomicU64,
}

impl VirtualClock {
    pub fn new(initial_time: SystemTime) -> Self {
        VirtualClock::from_value(
            initial_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
    }

    pub fn from_value(initial_value: u64) -> Self {
        VirtualClock {
            current_time: AtomicU64::new(initial_value),
            notify: Notify::new(),
            ticks_buff: AtomicU64::new(0),
        }
    }

    pub fn from_current_time() -> Self {
        Self::new(SystemTime::now())
    }

    pub fn from_epoch() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerClock<C> for VirtualClock {
    fn now(&self) -> SystemTime {
        let now = self.current_time.load(Ordering::Relaxed);
        UNIX_EPOCH + Duration::from_millis(now)
    }

    async fn idle_to(&self, to: SystemTime) {
        while <VirtualClock as SchedulerClock<C>>::now(self) < to {
            self.notify.notified().await;
        }
    }

    async fn tick(&self) {
        let res = self.ticks_buff.load(Ordering::Relaxed);
        if res > 0 {
            self.ticks_buff.fetch_sub(1, Ordering::Relaxed);
            return;
        }
        let prev = self.current_time.load(Ordering::Relaxed);
        self.notify.notified().await;
        let now = self.current_time.load(Ordering::Relaxed);
        self.ticks_buff.fetch_add((now - prev).saturating_sub(1), Ordering::Relaxed);
    }
}

#[async_trait]
impl<C: SchedulerConfig> AdvanceableSchedulerClock<C> for VirtualClock {
    fn advance(&self, duration: Duration) {
        let now = <VirtualClock as SchedulerClock<C>>::now(self);
        <VirtualClock as AdvanceableSchedulerClock<C>>::advance_to(self, now + duration)
    }

    fn advance_to(&self, to: SystemTime) {
        let to_millis = to
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.current_time.store(to_millis, Ordering::Relaxed);
        self.notify.notify_waiters();
    }
}
