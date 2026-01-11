use crate::scheduler::clock::SchedulerClock;
use async_trait::async_trait;
use std::time::SystemTime;
use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;

/// [`ProgressiveClock`] is an implementation of [`SchedulerClock`] trait, it is the default option
/// for scheduling, unlike [`VirtualClock`], it moves forward no matter what and cannot be advanced
/// at any arbitrary point (due to its design)
///
/// # Constructor(s)
/// One can simply use the default rust's struct initialization or via [`ProgressiveClock::default`]
/// to construct the [`ProgressiveClock`]
///
/// # Trait Implementation(s)
/// While [`ProgressiveClock`] implements the [`SchedulerClock`] trait, it also implements the
/// [`Default`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`VirtualClock`]
/// - [`SchedulerClock`]
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
