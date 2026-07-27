use crate::COUNTER;
use chronographer::errors::TaskError;
use chronographer::prelude::{DefaultSchedulerConfig, Scheduler, Task, TaskScheduleInterval};
use chronographer::scheduler::LiveScheduler;
use chronographer::task::TaskFrameContext;
use chronographer::taskframe;
use std::sync::LazyLock;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[taskframe]
async fn MyTaskFrame(_ctx: &TaskFrameContext) -> Result<(), Box<dyn TaskError>> {
    for _ in 0..100 {
        std::hint::black_box(42);
    }

    COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

static SCHEDULER: LazyLock<LiveScheduler<DefaultSchedulerConfig<Box<dyn TaskError>>>> =
    LazyLock::new(LiveScheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default);

pub async fn chronographer(tasks: usize, exec: Duration) {
    for _ in 0..tasks {
        let task = Task::new(MyTaskFrame, TaskScheduleInterval::duration(exec));

        let _ = SCHEDULER.schedule(task).await;
    }
}

pub async fn start_chronographer() {
    println!("LOADING SCHEDULER");
    SCHEDULER.start().await;
    println!("STARTING");
}
