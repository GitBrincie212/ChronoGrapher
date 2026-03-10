use std::sync::atomic::Ordering;
use async_trait::async_trait;
use tokio::task::yield_now;
use chronographer::errors::TaskError;
use chronographer::prelude::{DefaultSchedulerConfig, Scheduler, Task, TaskScheduleInterval};
use chronographer::task::{TaskFrame, TaskFrameContext};
use crate::COUNTER;

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

pub async fn benchmark_chronographer() {
    println!("LOADING TASKS");
    let t = tokio::time::Instant::now();
    let scheduler = Scheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default();

    for _ in 0..350_000 {
        let millis = fastrand::f64() / 6f64;
        let task = Task::new(TaskScheduleInterval::from_secs_f64(millis), MyTaskFrame);

        let _ = scheduler.schedule(&task).await;
    }

    scheduler.start().await;

    println!("STARTED {}", t.elapsed().as_secs_f64());
}