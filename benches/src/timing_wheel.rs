use chronographer::utils::{ByteWheel, HierarchicalTimingWheel};
use std::time::Duration;

#[divan::bench(args = [0u8, 1, 42, 128, 200, 255])]
fn insert_byte_wheel(bencher: divan::Bencher, pos: u8) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        wheel.insert(divan::black_box(pos), 1);
    });
}

#[divan::bench(args = [0u8, 1, 42, 128, 200, 255])]
fn skip_byte_wheel(bencher: divan::Bencher, to: u8) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        wheel.skip(divan::black_box(to));
    });
}

#[divan::bench]
fn tick_byte_wheel(bencher: divan::Bencher) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        divan::black_box(wheel.tick());
    });
}

#[divan::bench(args = [0usize, 10, 100, 1_000, 10_000])]
fn tick_byte_wheel_batch(bencher: divan::Bencher, count: usize) {
    bencher.counter(count).bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        for i in 0..count {
            wheel.insert(1, i);
        }
        let (expired, wrapped) = wheel.tick();
        divan::black_box((expired.len(), wrapped));
    });
}

#[divan::bench(args = [0usize, 10, 100, 1_000, 10_000])]
fn tick_byte_wheel_wrap(bencher: divan::Bencher, count: usize) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        for i in 0..count {
            wheel.insert(0, i);
        }
        wheel.skip(255);
        let (expired, wrapped) = wheel.tick();
        divan::black_box((expired.len(), wrapped));
    });
}

#[divan::bench(args = [0u64, 1, 100, 1_000, 60_000, 3_600_000, 86_400_000, 604_800_000])]
fn insert_htw(bencher: divan::Bencher, delay_ms: u64) {
    bencher.bench(|| {
        let mut wheel = HierarchicalTimingWheel::<u64>::default();
        wheel.insert(1, Duration::from_millis(divan::black_box(delay_ms)));
    });
}

#[divan::bench(args = [10u64, 100, 1_000, 10_000, 100_000])]
fn insert_htw_batch(bencher: divan::Bencher, count: u64) {
    bencher.counter(count).bench(|| {
        let mut wheel = HierarchicalTimingWheel::<u64>::default();
        for i in 0..count {
            wheel.insert(i, Duration::from_millis(i % 1_000_000));
        }
    });
}

#[divan::bench]
fn tick_htw(bencher: divan::Bencher) {
    bencher.bench(|| {
        let mut wheel = HierarchicalTimingWheel::<u64>::default();
        divan::black_box(wheel.tick());
    });
}

#[divan::bench(args = [0usize, 10, 100, 1_000, 10_000])]
fn tick_htw_cascade(bencher: divan::Bencher, count: usize) {
    bencher.counter(256u64).bench(|| {
        let mut wheel = HierarchicalTimingWheel::<u64>::default();
        for i in 0..count {
            wheel.insert(i as u64, Duration::from_millis(256));
        }
        for _ in 0..256 {
            divan::black_box(wheel.tick());
        }
    });
}

#[divan::bench(args = [0u64, 100, 10_000, 100_000])]
fn clear_htw(bencher: divan::Bencher, count: u64) {
    bencher.bench(|| {
        let mut wheel = HierarchicalTimingWheel::<u64>::default();
        for i in 0..count {
            wheel.insert(i, Duration::from_millis(i % 1_000_000));
        }
        wheel.clear();
    });
}
