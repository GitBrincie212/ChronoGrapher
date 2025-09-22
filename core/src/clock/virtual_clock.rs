use std::fmt::{Debug, Formatter};
use crate::clock::{AdvanceableScheduleClock, SchedulerClock};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;
use crate::utils::system_time_to_date_time;

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

impl Debug for VirtualClock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualClock")
            .field("current_time", &system_time_to_date_time(
                UNIX_EPOCH + Duration::from_millis(self.current_time.load(Ordering::Relaxed))
            ))
            .finish()
    }
}

impl VirtualClock {
    pub fn new(initial_time: SystemTime) -> Self {
        VirtualClock {
            current_time: AtomicU64::new(
                initial_time
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
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
        self.current_time
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.notify.notify_waiters();
    }

    async fn advance_to(&self, to: SystemTime) {
        let to_millis = to
            .duration_since(SystemTime::UNIX_EPOCH)
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

    async fn idle_to(&self, to: SystemTime) {
        while self.now().await < to {
            self.notify.notified().await;
        }
    }
}
