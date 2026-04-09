use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::time::Duration;
use chrono::Local;
use tokio::spawn;
use tokio_schedule::{every, Job};
use crate::COUNTER;

pub async fn tokio_schedule(tasks: usize, exec: Duration) {
    let mut prealloc: Vec<Pin<Box<dyn Future<Output=()> + Send>>> = Vec::with_capacity(tasks);

    for _ in 0..tasks {
        let t = every(exec.subsec_millis()).millisecond()
            .in_timezone(&Local)
            .perform(|| async {
                for _ in 0..100 {
                    std::hint::black_box(42);
                }

                COUNTER.fetch_add(1, Ordering::Relaxed);
            });

        prealloc.push(t);
    }

    for task in prealloc {
        spawn(task);
    }
}