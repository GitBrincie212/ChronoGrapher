use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;
use crate::scheduler::clock::SchedulerClock;

#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;

/// [`SystemClock`] is an implementation of [`SchedulerClock`] trait, it is the default option
/// for scheduling, unlike [`VirtualClock`], it moves forward no matter what and cannot be advanced
/// at any arbitrary point (due to its design)
///
/// # Constructor(s)
/// One can simply use the default rust's struct initialization or via [`SystemClock::default`]
/// to construct the [`SystemClock`]
///
/// # Trait Implementation(s)
/// While [`SystemClock`] implements the [`SchedulerClock`] trait, it also implements the
/// [`Default`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`VirtualClock`]
/// - [`SchedulerClock`]
#[derive(Default, Clone, Copy)]
pub struct SystemClock;

impl Debug for SystemClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SystemClock")
            .field(&SystemTime::now())
            .finish()
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
            Err(_) => {
                return;
            }
        };

        tokio::time::sleep(duration).await;
    }
}
