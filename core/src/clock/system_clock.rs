use std::fmt::Debug;
use crate::clock::SchedulerClock;
use async_trait::async_trait;
use std::time::{Duration, SystemTime};

#[allow(unused_imports)]
use crate::clock::VirtualClock;

/// [`SystemClock`] is an implementation of [`SchedulerClock`] trait, it is the default option
/// for scheduling, unlike [`VirtualClock`], it moves forward no matter what and cannot be advanced
/// at any arbitrary point (due to its design)
///
/// # See
/// - [`VirtualClock`]
/// - [`SchedulerClock`]
pub struct SystemClock;

impl Debug for SystemClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SystemClock").field(&SystemTime::now()).finish()
    }
}

#[async_trait]
impl SchedulerClock for SystemClock {
    async fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    async fn idle_to(&self, to: SystemTime) {
        let now = SystemTime::now();
        let duration = match to.duration_since(now) {
            Ok(duration) => duration,
            Err(diff) => {
                if diff.duration() <= Duration::from_millis(7) {
                    return;
                }
                panic!(
                    "Supposed future time is now in the past with a difference of {:?}",
                    diff.duration()
                );
            }
        };

        tokio::time::sleep(duration).await;
    }
}
