use crate::scheduler::clock::SchedulerClock;
use async_trait::async_trait;
use std::marker::PhantomData;

#[allow(unused_imports)]
use crate::scheduler::clock::VirtualClock;
use crate::utils::Timestamp;

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
pub struct ProgressiveClock<T: Timestamp>(PhantomData<T>);

impl<T: Timestamp> Default for ProgressiveClock<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Timestamp> Clone for ProgressiveClock<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T: Timestamp> Copy for ProgressiveClock<T> {}

#[async_trait]
impl<T: Timestamp> SchedulerClock<T> for ProgressiveClock<T> {
    async fn now(&self) -> T {
        T::now()
    }

    async fn idle_to(&self, to: T) {
        let now = T::now();
        let duration = match to.duration_since(now) {
            Some(duration) => duration,
            None => {
                return;
            }
        };

        tokio::time::sleep(duration).await;
    }
}
