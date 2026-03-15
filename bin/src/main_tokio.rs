use std::pin::Pin;
use std::sync::atomic::Ordering;
use chrono::Local;
use tokio::spawn;
use tokio::task::yield_now;
use tokio_schedule::{every, Job};
use crate::COUNTER;

pub async fn benchmark_tokio_schedule() {
    println!("LOADING TASKS");

    const EXEC_TIMES: usize = 6;
    const TASKS_ALLOCATED: usize = 450_000;

    let spread_millis = 1000.0 / ((TASKS_ALLOCATED * EXEC_TIMES) as f64);

    let mut tasks: Vec<Pin<Box<dyn Future<Output=()> + Send>>> = Vec::with_capacity(TASKS_ALLOCATED);

    let mut millis = 0f64;
    for _ in 0..TASKS_ALLOCATED {
        millis = (millis + spread_millis).rem_euclid(1000.0);
        let task = every(millis.floor() as u32).millisecond()
            .in_timezone(&Local)
            .perform(|| async {
                yield_now().await;
                COUNTER.fetch_add(1, Ordering::Relaxed);
            });
        tasks.push(task)
    }
    println!("STARTING");

    for task in tasks {
        spawn(task);
    }
}