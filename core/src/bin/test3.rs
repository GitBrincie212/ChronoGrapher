use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::scheduler::{DefaultSchedulerConfig, Scheduler};
use chronographer::task::{TaskFrame, TaskFrameContext};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::task::yield_now;

struct MyTaskFrame;

#[async_trait]
impl TaskFrame for MyTaskFrame {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        yield_now().await;
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
#[allow(clippy::empty_loop)]
async fn main() {
    let t = tokio::time::Instant::now();
    let scheduler = Scheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default();

    println!("LOADING TASKS");
    let mut millis: f64 = 0.9;
    for _ in 0..200_000 {
        millis *= 0.05;
        let task = Task::new(TaskScheduleInterval::from_secs_f64(millis), MyTaskFrame);

        let _ = scheduler.schedule(&task).await;
    }
    println!("STARTING {}", t.elapsed().as_secs_f64());

    scheduler.start().await;

    tokio::time::sleep(Duration::from_secs(50)).await;
}
