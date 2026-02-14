pub mod progressive_clock; // skipcq: RS-D1001

pub mod virtual_clock; // skipcq: RS-D1001

pub use progressive_clock::ProgressiveClock;
pub use virtual_clock::VirtualClock;

use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use std::ops::Deref;
use std::time::{Duration, SystemTime};

#[async_trait]
pub trait SchedulerClock<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn now(&self) -> SystemTime;

    async fn idle_to(&self, to: SystemTime);
}

#[async_trait]
impl<T, C: SchedulerConfig> SchedulerClock<C> for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: SchedulerClock<C>,
    C: SchedulerConfig,
{
    async fn now(&self) -> SystemTime {
        self.deref().now().await
    }

    async fn idle_to(&self, to: SystemTime) {
        self.deref().idle_to(to).await
    }
}

#[async_trait]
pub trait AdvanceableSchedulerClock<C: SchedulerConfig>: SchedulerClock<C> {
    async fn advance(&self, duration: Duration);

    async fn advance_to(&self, to: SystemTime);
}

#[async_trait]
impl<T, C> AdvanceableSchedulerClock<C> for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: AdvanceableSchedulerClock<C>,
    C: SchedulerConfig,
{
    async fn advance(&self, duration: Duration) {
        self.deref().advance(duration).await
    }

    async fn advance_to(&self, to: SystemTime) {
        self.deref().advance_to(to).await
    }
}
