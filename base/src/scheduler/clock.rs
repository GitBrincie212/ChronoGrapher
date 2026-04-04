pub mod progressive_clock; // skipcq: RS-D1001

pub mod virtual_clock; // skipcq: RS-D1001

pub use progressive_clock::ProgressiveClock;
pub use virtual_clock::VirtualClock;

use async_trait::async_trait;
use std::ops::Deref;
use std::time::{Duration, SystemTime};

#[async_trait]
pub trait SchedulerClock: 'static + Send + Sync {
    fn now(&self) -> SystemTime;

    async fn idle_to(&self, to: SystemTime);

    fn tick(&self)  -> impl Future<Output = ()> + Send;
}

#[async_trait]
impl<T> SchedulerClock for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: SchedulerClock,
{
    fn now(&self) -> SystemTime {
        self.deref().now()
    }

    async fn idle_to(&self, to: SystemTime) {
        self.deref().idle_to(to).await
    }

    fn tick(&self) -> impl Future<Output = ()> + Send {
        self.deref().tick()
    }
}

#[async_trait]
pub trait AdvanceableSchedulerClock: SchedulerClock {
    fn advance(&self, duration: Duration);

    fn advance_to(&self, to: SystemTime);
}

#[async_trait]
impl<T> AdvanceableSchedulerClock for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: AdvanceableSchedulerClock,
{
    fn advance(&self, duration: Duration) {
        self.deref().advance(duration)
    }

    fn advance_to(&self, to: SystemTime) {
        self.deref().advance_to(to)
    }
}
