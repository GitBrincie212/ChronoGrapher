use crate::util::{noop_erased, BenchConfig};
use chronographer::scheduler::task_store::{EphemeralSchedulerTaskStore, SchedulerTaskStore};

type Store = EphemeralSchedulerTaskStore<BenchConfig>;

#[divan::bench]
fn insert(bencher: divan::Bencher) {
    let store = Store::default();
    let task = noop_erased();
    bencher.bench(|| {
        divan::black_box(store.store(task.clone()).unwrap());
    });
}

#[divan::bench]
fn get(bencher: divan::Bencher) {
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    bencher.bench(|| {
        divan::black_box(store.get(&key).is_some());
    });
}

#[divan::bench]
fn exists(bencher: divan::Bencher) {
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    bencher.bench(|| {
        divan::black_box(store.exists(&key));
    });
}

#[divan::bench]
fn remove(bencher: divan::Bencher) {
    let store = Store::default();
    bencher.bench(|| {
        let key = store.store(noop_erased()).unwrap();
        store.remove(&key);
    });
}

#[divan::bench(args = [10u64, 100, 1_000, 10_000])]
fn insert_batch(bencher: divan::Bencher, count: u64) {
    let store = Store::default();
    let task = noop_erased();
    bencher.counter(count).bench(|| {
        for _ in 0..count {
            store.store(task.clone()).unwrap();
        }
    });
}

#[divan::bench(args = [10usize, 100, 1_000, 10_000])]
fn get_batch(bencher: divan::Bencher, count: usize) {
    let store = Store::default();
    let task = noop_erased();
    let keys: Vec<_> = (0..count).map(|_| store.store(task.clone()).unwrap()).collect();
    bencher.counter(count as u64).bench(|| {
        for key in &keys {
            divan::black_box(store.get(key).is_some());
        }
    });
}

#[divan::bench(args = [10usize, 100, 1_000, 10_000])]
fn exists_batch(bencher: divan::Bencher, count: usize) {
    let store = Store::default();
    let task = noop_erased();
    let keys: Vec<_> = (0..count).map(|_| store.store(task.clone()).unwrap()).collect();
    bencher.counter(count as u64).bench(|| {
        for key in &keys {
            divan::black_box(store.exists(key));
        }
    });
}

#[divan::bench(args = [10usize, 100, 1_000, 10_000])]
fn remove_batch(bencher: divan::Bencher, count: usize) {
    let store = Store::default();
    let task = noop_erased();
    let keys: Vec<_> = (0..count).map(|_| store.store(task.clone()).unwrap()).collect();
    bencher.counter(count as u64).bench(|| {
        for key in &keys {
            store.remove(key);
        }
    });
}

#[divan::bench(args = [0usize, 100, 10_000, 100_000])]
fn clear(bencher: divan::Bencher, count: usize) {
    bencher.counter(count).bench(|| {
        let store = Store::default();
        let task = noop_erased();
        for _ in 0..count {
            store.store(task.clone()).unwrap();
        }
        store.clear();
    });
}
