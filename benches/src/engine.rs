use crate::util::{noop_erased, runtime, BenchConfig};
use chronographer::scheduler::clock::SchedulerClock;
use chronographer::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine};
use chronographer::scheduler::task_store::{EphemeralSchedulerTaskStore, SchedulerTaskStore};
use std::time::Duration;

type Engine = DefaultSchedulerEngine<BenchConfig>;
type Store = EphemeralSchedulerTaskStore<BenchConfig>;

#[divan::bench(args = [Duration::from_secs(0), Duration::from_millis(1), Duration::from_secs(1), Duration::from_secs(60), Duration::from_secs(3600)])]
fn schedule(bencher: divan::Bencher, delay: Duration) {
    let engine = Engine::default();
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    let time = engine.clock().now() + delay;
    bencher.bench(|| {
        runtime()
            .block_on(engine.schedule(&key, divan::black_box(time)))
            .unwrap();
    });
}

#[divan::bench(args = [10u64, 100, 1_000])]
fn schedule_batch(bencher: divan::Bencher, count: u64) {
    let engine = Engine::default();
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    let time = engine.clock().now() + Duration::from_secs(3600);
    bencher.counter(count).bench(|| {
        runtime().block_on(async {
            for _ in 0..count {
                engine.schedule(&key, time).await.unwrap();
            }
        });
    });
}

#[divan::bench]
fn clear(bencher: divan::Bencher) {
    let engine = Engine::default();
    bencher.bench(|| {
        runtime().block_on(engine.clear());
    });
}
