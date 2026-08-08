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

