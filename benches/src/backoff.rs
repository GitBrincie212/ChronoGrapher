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

