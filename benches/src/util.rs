use async_trait::async_trait;
use chronographer::scheduler::clock::{AdvanceableSchedulerClock, SchedulerClock, VirtualClock};
use chronographer::scheduler::engine::DefaultSchedulerEngine;
use chronographer::scheduler::task_dispatcher::DefaultTaskDispatcher;
use chronographer::scheduler::task_store::EphemeralSchedulerTaskStore;
use chronographer::scheduler::{LiveScheduler, SchedulerConfig};
use chronographer::task::{
    NoOperationTaskFrame, OnTaskEnd, Task, TaskHook, TaskHookContext, TaskHookEvent, TaskSchedule,
    TaskScheduleImmediate,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

pub fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("failed to create the benchmark Tokio runtime"))
}

pub type BenchError = String;

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

pub fn advance_clock(ms: u64) {
    let clock = SharedVirtualClock::default();
    clock.advance_to(UNIX_EPOCH + Duration::from_millis(ms));
}

pub struct BenchConfig;

impl SchedulerConfig for BenchConfig {
    type TaskError = BenchError;

    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher<Self>;
    type SchedulerEngine = DefaultSchedulerEngine<Self>;
    type SchedulerClock = SharedVirtualClock;
}

pub type BenchScheduler = LiveScheduler<BenchConfig>;

pub type NoopFrame = NoOperationTaskFrame<BenchError>;

pub fn noop_task(schedule: impl TaskSchedule + 'static) -> Task<NoOperationTaskFrame<BenchError>> {
    Task::new(NoOperationTaskFrame::<BenchError>::default(), schedule)
}

pub fn noop_erased() -> Arc<chronographer::task::ErasedTask<BenchError>> {
    Arc::new(noop_task(TaskScheduleImmediate).into_erased())
}

pub struct TaskCompletionCountdown(Arc<AtomicUsize>);

impl TaskCompletionCountdown {
    pub fn new(count: Arc<AtomicUsize>) -> Self {
        Self(count)
    }
}

#[async_trait]
impl TaskHook<OnTaskEnd> for TaskCompletionCountdown {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>,
    ) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn integration_enabled() -> bool {
    std::env::var("CHRONO_BENCH_INTEGRATION").as_deref() == Ok("true")
}
