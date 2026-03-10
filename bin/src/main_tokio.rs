use std::pin::Pin;
use std::sync::atomic::Ordering;
use chrono::Local;
use tokio::spawn;
use tokio::task::yield_now;
use tokio_schedule::{every, Job};
use crate::COUNTER;

pub async fn benchmark_tokio_schedule() {
    println!("LOADING TASKS");
    let mut tasks: Vec<Pin<Box<dyn Future<Output=()> + Send>>> = Vec::with_capacity(200_000);
    for _ in 0..350_000 {
        let millis = fastrand::f64() / 6f64;
        let task = every((millis * 1000.0).floor() as u32).millisecond()
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