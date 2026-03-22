use crate::main_cg::benchmark_chronographer;
use std::io::Write;
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::Duration;
use crate::main_tokio::benchmark_tokio_schedule;

mod main_cg;
mod main_tokio;

pub static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub async fn benchmark() {
    let mut last = COUNTER.load(Ordering::Relaxed);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("tasks_per_sec.csv")
        .unwrap();

    writeln!(file, "time_sec,tasks_per_sec").unwrap();

    for i in 0..=50 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let delta = COUNTER.swap(0, Ordering::SeqCst);

        println!("{}", i);
        writeln!(file, "{:.2},{:.2}", i, delta).unwrap();
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
#[allow(clippy::empty_loop)]
async fn main() {
    benchmark_chronographer().await;
    benchmark().await;
}