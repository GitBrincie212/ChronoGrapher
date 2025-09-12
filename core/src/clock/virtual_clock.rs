use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use tokio::sync::Notify;
use crate::clock::{AdvanceableScheduleClock, SchedulerClock};

/// [`VirtualClock`] is an implementation of the [`SchedulerClock`] trait, it acts as a mock object, allowing
/// to simulate time without the waiting around. This can especially be useful for unit tests,
/// simulations of a [`flashcrowd`](https://en.wiktionary.org/wiki/flashcrowd#English)
///
/// Unlike [`SystemClock`], this clock doesn't move forward, rather it needs explicit
/// calls to advance methods ([`VirtualClock`] implements the [`AdvanceableScheduleClock`] extension
/// trait), which makes it predictable at any point throughout the program
///
/// # See
/// - [`SystemClock`]
/// - [`AdvanceableScheduleClock`]
/// - [`SchedulerClock`]
pub struct VirtualClock {
    current_time: AtomicU64,
    notify: Notify,
}

impl VirtualClock {
    pub fn new(initial_time: SystemTime) -> Self {
        VirtualClock {
            current_time: AtomicU64::new(
                initial_time
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            ),
            notify: Notify::new(),
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
impl AdvanceableScheduleClock for VirtualClock {
    async fn advance(&self, duration: Duration) {
        self.current_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.notify.notify_waiters();
    }

    async fn advance_to(&self, to: SystemTime) {
        let to_millis = to.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.current_time.fetch_add(to_millis, Ordering::Relaxed);
        self.notify.notify_waiters();
    }
}

#[async_trait]
impl SchedulerClock for VirtualClock {
    async fn now(&self) -> SystemTime {
        let now = self.current_time.load(Ordering::Relaxed);
        UNIX_EPOCH + Duration::from_millis(now)
    }

    async fn idle(&self, duration: Duration) {
        self.idle_to(self.now().await + duration).await;
    }

    async fn idle_to(&self, to: SystemTime) {
        while self.now().await < to {
            self.notify.notified().await;
        }
    }
}