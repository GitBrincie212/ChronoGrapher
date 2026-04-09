/*
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use async_trait::async_trait;
use tokio::task::yield_now;
use chronographer::errors::TaskError;
use chronographer::prelude::{DefaultSchedulerConfig, Scheduler, Task, TaskFrameBuilder, TaskHook, TaskHookEvent, TaskScheduleInterval};
use chronographer::task::{NoOperationTaskFrame, OnTaskEnd, OnTaskStart, OnTimeout, TaskFrame, TaskFrameContext, TaskHookContext};
use crate::COUNTER;

struct MyTaskFrame;

#[async_trait]
impl TaskFrame for MyTaskFrame {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        yield_now().await;
        COUNTER.fetch_add(1, Ordering::Relaxed);
        if fastrand::bool() {
            return Err(Box::new("Hello World") as Box<dyn TaskError>);
        }
        Ok(())
    }
}

struct MyDummyHook(Vec<u8>);

#[async_trait]
impl TaskHook<OnTaskEnd> for MyDummyHook {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>) {
        yield_now().await;
    }
}

#[async_trait]
impl TaskHook<OnTaskStart> for MyDummyHook {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>) {
        yield_now().await;
    }
}

#[async_trait]
impl TaskHook<OnTimeout> for MyDummyHook {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &<OnTimeout as TaskHookEvent>::Payload<'_>) {
        yield_now().await;
    }
}

pub async fn benchmark_chronographer() {
    println!("LOADING TASKS");
    let t = tokio::time::Instant::now();
    let scheduler = Scheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default();

    for _ in 0..350_000 {
        let millis = fastrand::f64() / 6f64;
        let task = Task::new(
            TaskScheduleInterval::from_secs_f64(millis),
            TaskFrameBuilder::new(MyTaskFrame)
                .with_timeout(Duration::from_secs_f64(31.234))
                .with_fallback(NoOperationTaskFrame::<Box<dyn TaskError>>::default())
                .with_instant_retry(NonZeroU32::new(3).unwrap())
                .with_timeout(Duration::from_secs_f64(30.5))
                .with_fallback(NoOperationTaskFrame::default())
                .build()
        );

        task.attach_hook::<OnTaskStart>(Arc::new(MyDummyHook(Vec::with_capacity(1024)))).await;
        task.attach_hook::<OnTaskEnd>(Arc::new(MyDummyHook(Vec::with_capacity(596)))).await;
        task.attach_hook::<OnTimeout>(Arc::new(MyDummyHook(Vec::with_capacity(392)))).await;

        let _ = scheduler.schedule(&task).await;
    }

    scheduler.start().await;

    println!("STARTED {}", t.elapsed().as_secs_f64());
}
 */
use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::time::Duration;
use async_trait::async_trait;
use chronographer::errors::TaskError;
use chronographer::prelude::{DefaultSchedulerConfig, Scheduler, Task, TaskScheduleInterval};
use chronographer::task::{TaskFrame, TaskFrameContext};
use crate::COUNTER;

struct MyTaskFrame;

#[async_trait]
impl TaskFrame for MyTaskFrame {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        for _ in 0..100 {
            std::hint::black_box(42);
        }

        COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

static SCHEDULER: LazyLock<Scheduler<DefaultSchedulerConfig<Box<dyn TaskError>>>> =
    LazyLock::new(|| Scheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default());

pub async fn chronographer(tasks: usize, exec: Duration) {
    for _ in 0..tasks {
        let task = Task::new(
            TaskScheduleInterval::duration(exec),
            MyTaskFrame
        );

        let _ = SCHEDULER.schedule(task).await;
    }
}

pub async fn start_chronographer() {
    println!("LOADING SCHEDULER");
    SCHEDULER.start().await;
    println!("STARTING");
}