use chronographer::scheduler::clock::{AdvanceableSchedulerClock, SchedulerClock, VirtualClock};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;
use tokio::runtime::Runtime;

pub fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("failed to create the benchmark Tokio runtime"))
}

#[derive(Clone)]
pub struct SharedVirtualClock(Arc<VirtualClock>);

impl Default for SharedVirtualClock {
    fn default() -> Self {
        static CLOCK: OnceLock<Arc<VirtualClock>> = OnceLock::new();
        SharedVirtualClock(
            CLOCK
                .get_or_init(|| Arc::new(VirtualClock::from_epoch()))
                .clone(),
        )
    }
}

impl SchedulerClock for SharedVirtualClock {
    fn now(&self) -> SystemTime {
        self.0.now()
    }

    async fn idle_to(&self, to: SystemTime) {
        self.0.idle_to(to).await;
    }

    async fn tick(&self) {
        self.0.tick().await;
    }
}

impl AdvanceableSchedulerClock for SharedVirtualClock {
    fn advance_to(&self, to: SystemTime) {
        self.0.advance_to(to);
    }
}

