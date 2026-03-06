use crate::main_cg::benchmark_chronographer;
use std::io::Write;
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

mod main_cg;
mod main_tokio;

pub static COUNTER: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub async fn benchmark() {
    let mut last = COUNTER.load(Ordering::Relaxed);
    let mut total = 0usize;
    let start = Instant::now();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("tasks_per_sec.csv")
        .unwrap();

    writeln!(file, "time_sec,tasks_per_sec").unwrap();

    for i in 0..=50 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let current = COUNTER.load(Ordering::Relaxed);
        let delta = current - last;
        last = current;

        total += delta;

        let elapsed = start.elapsed().as_secs_f64();
        let avg = total as f64 / elapsed;

        println!("{}", i);
        writeln!(file, "{:.2},{:.2}", elapsed, avg).unwrap();
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
#[allow(clippy::empty_loop)]
async fn main() {
    benchmark_chronographer().await;
    benchmark().await;
}