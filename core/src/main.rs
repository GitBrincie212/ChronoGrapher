use chronographer::dynamic_taskframe;
use chronographer::prelude::*;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::task::yield_now;

static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[tokio::main]
#[allow(clippy::empty_loop)]
async fn main() {
    dbg!("LOADING TASKS");
    let mut millis: f64 = 0.9;
    for _ in 0..200_000 {
        millis *= 0.05;
        let task = Task::simple(
            TaskScheduleInterval::from_secs_f64(millis),
            dynamic_taskframe!({
                // println!("{}", INSTANT.elapsed().as_secs_f64());
                yield_now().await;
                COUNTER.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }),
        );

        CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
    }
    dbg!("STARTING");

    CHRONOGRAPHER_SCHEDULER.start().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    CHRONOGRAPHER_SCHEDULER.abort().await;
    println!("{}", COUNTER.load(Ordering::Relaxed));
}
