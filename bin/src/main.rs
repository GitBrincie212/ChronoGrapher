use crate::main_cg::{chronographer, start_chronographer};
use std::io::Write;
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::Duration;

mod main_cg;
mod main_tokio;
mod main_pure;

pub const TASK_BATCH: usize = 1_000;
pub const EXEC_TIME: Duration = Duration::from_millis(2);

pub static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    chronographer(TASK_BATCH, EXEC_TIME).await;
    start_chronographer().await;
    let mut last = COUNTER.load(Ordering::Relaxed);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("tasks_per_sec.csv")
        .unwrap();

    writeln!(file, "time_sec,tasks_per_sec").unwrap();

    let mut i = 0usize;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let curr = COUNTER.load(Ordering::Relaxed);
        let delta = curr.abs_diff(last);

        println!("{i}. {delta}");
        writeln!(file, "{:.2},{:.2}", i, delta).unwrap();
        i += 1;

        chronographer(TASK_BATCH, EXEC_TIME).await;
        tokio::time::sleep(Duration::from_secs_f64(0.1)).await;
        last = COUNTER.load(Ordering::Relaxed);
    }
}