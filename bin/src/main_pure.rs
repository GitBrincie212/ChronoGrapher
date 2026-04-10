use std::sync::atomic::Ordering;
use std::time::Duration;
use crate::COUNTER;

pub async fn pure_tokio(tasks: usize, exec: Duration) {
    for _ in 0..tasks {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(exec);

            loop {
                interval.tick().await;
                for _ in 0..100 {
                    std::hint::black_box(42);
                }

                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        });
    }
}