pub mod progressive_clock; // skipcq: RS-D1001

pub mod virtual_clock; // skipcq: RS-D1001

pub use progressive_clock::ProgressiveClock;
pub use virtual_clock::VirtualClock;

use std::time::{Duration, SystemTime};

pub trait SchedulerClock: 'static + Send + Sync {
    fn now(&self) -> SystemTime;

    fn idle_to(&self, to: SystemTime) -> impl Future<Output = ()> + Send;

    fn tick(&self) -> impl Future<Output = ()> + Send;
}

pub trait AdvanceableSchedulerClock: SchedulerClock {
    fn advance(&self, duration: Duration) {
        let now = self.now();
        self.advance_to(now + duration);
    }

    fn advance_to(&self, to: SystemTime);
}