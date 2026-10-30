use chronographer::task::{
    ConstantBackoffStrategy, ExponentialBackoffStrategy, JitterBackoffStrategy,
    LinearBackoffStrategy, RetryBackoffStrategy,
};
use std::time::Duration;

const RETRIES: &[u32] = &[0, 1, 2, 3, 4, 8, 16, 32];

#[divan::bench(args = RETRIES)]
fn constant(bencher: divan::Bencher, retry: u32) {
    let strategy = ConstantBackoffStrategy::new(Duration::from_secs(1));
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn linear(bencher: divan::Bencher, retry: u32) {
    let strategy = LinearBackoffStrategy::builder()
        .factor(Duration::from_millis(100))
        .start(Duration::from_millis(10))
        .build();
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn linear_bounded(bencher: divan::Bencher, retry: u32) {
    let strategy = LinearBackoffStrategy::builder()
        .factor(Duration::from_millis(100))
        .clamp(Duration::from_secs(30))
        .build();
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn exponential(bencher: divan::Bencher, retry: u32) {
    let strategy = ExponentialBackoffStrategy::new(2.0);
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn exponential_bounded(bencher: divan::Bencher, retry: u32) {
    let strategy = ExponentialBackoffStrategy::new_with(2.0, Duration::from_secs(30));
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn jitter_full(bencher: divan::Bencher, retry: u32) {
    let strategy = JitterBackoffStrategy::full(
        ConstantBackoffStrategy::new(Duration::from_millis(100)),
        2.0,
    );
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn jitter_equal(bencher: divan::Bencher, retry: u32) {
    let strategy = JitterBackoffStrategy::equal(
        ConstantBackoffStrategy::new(Duration::from_millis(100)),
        2.0,
    );
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = RETRIES)]
fn jitter_decorrelated(bencher: divan::Bencher, retry: u32) {
    let strategy = JitterBackoffStrategy::decorrelated(
        ConstantBackoffStrategy::new(Duration::from_millis(100)),
        2.0,
        30.0,
    );
    bencher.bench(|| {
        divan::black_box(strategy.compute(divan::black_box(retry)));
    });
}

#[divan::bench(args = [1_000u64, 10_000, 100_000, 1_000_000])]
fn compute_batch(bencher: divan::Bencher, count: u64) {
    let strategy = ConstantBackoffStrategy::new(Duration::from_millis(1));
    bencher.counter(count).bench(|| {
        let mut total = Duration::ZERO;
        for i in 0..count {
            total += strategy.compute(divan::black_box(i as u32));
        }
        divan::black_box(total);
    });
}
