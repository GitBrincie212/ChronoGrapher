use chrono::{Local, Timelike};
use chronographer_core::schedule::TaskScheduleInterval;
use chronographer_core::scheduler::CHRONOGRAPHER_SCHEDULER;
use chronographer_core::task::{ExecutionTaskFrame, Task};
use rand::Rng;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let now = Local::now();
    println!(
        "PROCESS STARTED {}.{}s",
        now.second(),
        now.timestamp_subsec_millis()
    );

    for _ in 0..10_000 {
        let rand1 = rand::rng().random_range(0..5000);
        let rand2 = rand::rng().random_range(0..5000);
        let frame = ExecutionTaskFrame::new(move |_| async move {
            tokio::time::sleep(Duration::from_millis(rand1)).await;
            Ok(())
        });

        let task = Task::define(
            TaskScheduleInterval::duration(Duration::from_millis(rand2)),
            frame,
        );

        CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
    }
    CHRONOGRAPHER_SCHEDULER.start().await;
    loop {}
}
