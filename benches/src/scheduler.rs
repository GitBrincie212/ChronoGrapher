use crate::util::{
    BenchScheduler, TaskCompletionCountdown, advance_clock, integration_enabled, noop_task, runtime,
};
use chronographer::scheduler::Scheduler;
use chronographer::task::{OnTaskEnd, TaskScheduleImmediate, TaskScheduleInterval};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[divan::bench]
fn submit(bencher: divan::Bencher) {
    let scheduler = BenchScheduler::default();
    bencher.bench(|| {
        runtime()
            .block_on(scheduler.schedule(noop_task(TaskScheduleInterval::duration(
                Duration::from_secs(3600),
            ))))
            .unwrap();
    });
}

#[divan::bench(args = [10u64, 100, 1_000, 10_000])]
fn submit_batch(bencher: divan::Bencher, count: u64) {
    let scheduler = BenchScheduler::default();
    bencher.counter(count).bench(|| {
        runtime().block_on(async {
            for _ in 0..count {
                scheduler
                    .schedule(noop_task(TaskScheduleInterval::duration(
                        Duration::from_secs(3600),
                    )))
                    .await
                    .unwrap();
            }
        });
    });
}

#[divan::bench(args = [10usize, 100, 1_000, 10_000])]
fn exists_throughput(bencher: divan::Bencher, count: usize) {
    let scheduler = BenchScheduler::default();
    let keys: Vec<_> = runtime().block_on(async {
        let mut keys = Vec::with_capacity(count);
        for _ in 0..count {
            keys.push(
                scheduler
                    .schedule(noop_task(TaskScheduleInterval::duration(
                        Duration::from_secs(3600),
                    )))
                    .await
                    .unwrap(),
            );
        }
        keys
    });
    bencher.counter(count as u64).bench(|| {
        for key in &keys {
            runtime().block_on(scheduler.exists(key));
        }
    });
}

#[divan::bench(args = [10usize, 100, 1_000, 10_000])]
fn remove_throughput(bencher: divan::Bencher, count: usize) {
    let scheduler = BenchScheduler::default();
    let keys: Vec<_> = runtime().block_on(async {
        let mut keys = Vec::with_capacity(count);
        for _ in 0..count {
            keys.push(
                scheduler
                    .schedule(noop_task(TaskScheduleInterval::duration(
                        Duration::from_secs(3600),
                    )))
                    .await
                    .unwrap(),
            );
        }
        keys
    });
    bencher.counter(count as u64).bench(|| {
        for key in &keys {
            runtime().block_on(scheduler.remove(key));
        }
    });
}

#[divan::bench(args = [0usize, 100, 10_000, 100_000])]
fn clear_throughput(bencher: divan::Bencher, count: usize) {
    let scheduler = BenchScheduler::default();
    bencher.counter(count).bench(|| {
        runtime().block_on(async {
            for _ in 0..count {
                scheduler
                    .schedule(noop_task(TaskScheduleInterval::duration(
                        Duration::from_secs(3600),
                    )))
                    .await
                    .unwrap();
            }
            scheduler.clear().await;
        });
    });
}

// gated behind the `CHRONO_BENCH_INTEGRATION=true` environment variable
#[divan::bench(args = [10usize, 100, 1_000])]
fn integration(bencher: divan::Bencher, count: usize) {
    if !integration_enabled() {
        return;
    }

    let scheduler = BenchScheduler::default();
    runtime().block_on(scheduler.start());

    bencher.bench(|| {
        let done = Arc::new(AtomicUsize::new(0));
        runtime().block_on(async {
            for _ in 0..count {
                let task = noop_task(TaskScheduleImmediate);
                task.attach_hook::<OnTaskEnd>(Arc::new(TaskCompletionCountdown::new(done.clone())))
                    .await;
                scheduler.schedule(task).await.unwrap();
            }

            let mut i = 0u64;
            while done.load(Ordering::Relaxed) < count && i < 10_000 {
                advance_clock(i);
                i += 1;
                tokio::task::yield_now().await;
            }
        });
        divan::black_box(done.load(Ordering::Relaxed));
    });
}
