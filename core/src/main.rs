/*
use std::io::Write;
use std::fs::OpenOptions;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::Duration;
use chrono::{Local};
use tokio::spawn;
use tokio::task::yield_now;
use tokio::time::Instant;
use tokio_schedule::{every, Job};

static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[tokio::main]
async fn main() {
    dbg!("LOADING TASKS");
    let mut millis: f64 = 0.9;
    let mut tasks: Vec<Pin<Box<dyn Future<Output=()> + Send>>> = Vec::with_capacity(200_000);
    for _ in 0..200_000 {
        millis *= 0.05;
        let task = every((millis * 1000.0).floor() as u32).millisecond()
            .in_timezone(&Local)
            .perform(|| async {
                yield_now().await;
                COUNTER.fetch_add(1, Ordering::Relaxed);
            });
        tasks.push(task)
    }
    dbg!("STARTING");

    for task in tasks {
        spawn(task);
    }

    let mut last = COUNTER.load(Ordering::Relaxed);
    let mut total = 0usize;
    let start = Instant::now();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("tasks_per_sec.csv")
        .unwrap();

    writeln!(file, "time_sec,tasks_per_sec").unwrap();

    for _ in 0..=50 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let current = COUNTER.load(Ordering::Relaxed);
        let delta = current - last;
        last = current;

        total += delta;

        let elapsed = start.elapsed().as_secs_f64();
        let avg = total as f64 / elapsed;

        println!("Average tasks/sec: {:.2}", avg);
        writeln!(file, "{:.2},{:.2}", elapsed, avg).unwrap();
    }
}
*/
use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::TaskFrame;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::task::yield_now;

struct MyTaskFrame;

#[async_trait]
impl TaskFrame for MyTaskFrame {
    async fn execute(&self, _ctx: &TaskContext) -> Result<(), TaskError> {
        yield_now().await;
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[tokio::main]
#[allow(clippy::empty_loop)]
async fn main() {
    dbg!("LOADING TASKS");
    let mut millis: f64 = 0.9;
    for _ in 0..200_000 {
        millis *= 0.05;
        let task = Task::simple(TaskScheduleInterval::from_secs_f64(millis), MyTaskFrame);

        CHRONOGRAPHER_SCHEDULER
            .schedule(&task)
            .await
            .expect("Failed to schedule task");
    }
    dbg!("STARTING");

    CHRONOGRAPHER_SCHEDULER.start().await;
    let mut last = COUNTER.load(Ordering::Relaxed);
    let mut total = 0usize;
    let start = Instant::now();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("tasks_per_sec.csv")
        .unwrap();

    writeln!(file, "time_sec,tasks_per_sec").unwrap();

    for _ in 0..=50 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let current = COUNTER.load(Ordering::Relaxed);
        let delta = current - last;
        last = current;

        total += delta;

        let elapsed = start.elapsed().as_secs_f64();
        let avg = total as f64 / elapsed;

        println!("Average tasks/sec: {:.2}", avg);
        writeln!(file, "{:.2},{:.2}", elapsed, avg).unwrap();
    }
}
