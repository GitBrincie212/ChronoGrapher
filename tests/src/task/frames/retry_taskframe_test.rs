use chronographer::task::{
    ConstantBackoffStrategy, ExponentialBackoffStrategy, JitterBackoffStrategy,
    LinearBackoffStrategy, RetriableTaskFrame, Task, TaskFrame,
    TaskFrameContext, TaskScheduleImmediate,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

macro_rules! retry_frame_builder {
    ($frame:expr, retries = $retries:expr, when = $when:expr) => {
        RetriableTaskFrame::builder()
            .frame($frame)
            .retries(NonZeroU32::new($retries).unwrap())
            .constant(Duration::ZERO)
            .when($when)
            .build()
    };
    ($frame:expr, retries = $retries:expr) => {
        RetriableTaskFrame::builder()
            .frame($frame)
            .retries(NonZeroU32::new($retries).unwrap())
            .constant(Duration::ZERO)
            .build()
    };
}

struct FailNTimesFrame {
    counter: Arc<AtomicUsize>,
    fail_times: usize,
}

impl TaskFrame for FailNTimesFrame {
    type Error = String;
    type Args = ();
    type Workflow = Self;

    async fn execute(&self, _ctx: &TaskFrameContext, _args: &Self::Args) -> Result<(), Self::Error> {
        let attempt = self.counter.fetch_add(1, Ordering::SeqCst);
        if attempt < self.fail_times {
            return Err("frame failed".to_string());
        }
        Ok(())
    }
}

macro_rules! backoff_spawn {
    ($frame:expr) => {
        tokio::spawn(async move {
            Task::new($frame, TaskScheduleImmediate).into_erased().run().await
        })
    };
}

const NS: Duration = Duration::from_nanos(1);

#[tokio::test]
async fn retry_succeeds_eventually() {
    let counter = Arc::new(AtomicUsize::new(0));
    let fail_times = 2;

    let frame = retry_frame_builder!(FailNTimesFrame { counter: counter.clone(), fail_times }, retries = 3);

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "should succeed after retries");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        fail_times + 1,
        "should attempt exactly fail_times + 1 times"
    );
}


#[tokio::test]
async fn retry_exhausted_returns_err() {
    let counter = Arc::new(AtomicUsize::new(0));
    let retries = 3u32;

    let frame = retry_frame_builder!(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX }, retries = retries);

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_err(), "should fail after retries exhausted");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        retries as usize + 1,
        "should attempt retries + 1 times total"
    );
}

#[tokio::test]
async fn retry_skipped_on_success() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = retry_frame_builder!(FailNTimesFrame { counter: counter.clone(), fail_times: 0 }, retries = 3);

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "should succeed on first attempt");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "should only attempt once"
    );
}

#[tokio::test]
async fn retry_when_filter_stops_retry() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = retry_frame_builder!(
        FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX },
        retries = 3,
        when = |_err: Option<&String>| async { false }
    );

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "when=false stops retry and swallows error");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "should only attempt once before filter stops retry"
    );
}

#[tokio::test]
async fn retry_when_filter_always_true_exhausts_retries() {
    let counter = Arc::new(AtomicUsize::new(0));
    let retries = 3u32;

    let frame = retry_frame_builder!(
        FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX },
        retries = retries,
        when = |_err: Option<&String>| async { true }
    );

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_err(), "when=true should not suppress retries");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        retries as usize + 1,
        "should attempt retries + 1 times"
    );
}

struct SelectiveErrorFrame {
    counter: Arc<AtomicUsize>,
    stop_at: usize,
}

impl TaskFrame for SelectiveErrorFrame {
    type Error = String;
    type Args = ();
    type Workflow = Self;

    async fn execute(&self, _ctx: &TaskFrameContext, _args: &Self::Args) -> Result<(), Self::Error> {
        let attempt = self.counter.fetch_add(1, Ordering::SeqCst);
        if attempt == self.stop_at {
            Err("stop".to_string())
        } else {
            Err("retry".to_string())
        }
    }
}

#[tokio::test]
async fn retry_when_filter_inspects_error_value_stops_on_match() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = retry_frame_builder!(
        SelectiveErrorFrame { counter: counter.clone(), stop_at: 1 },
        retries = 5,
        when = |err: Option<&String>| {
            let should_retry = err.map(|e| e == "retry").unwrap_or(false);
            async move { should_retry }
        }
    );

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "filter returning false should swallow error and return Ok");
    assert_eq!(counter.load(Ordering::SeqCst), 2, "should stop after hitting the non-retriable error");
}

#[tokio::test]
async fn retry_when_filter_retries_past_matching_errors() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = retry_frame_builder!(
        SelectiveErrorFrame { counter: counter.clone(), stop_at: 3 },
        retries = 5,
        when = |err: Option<&String>| {
            let should_retry = err.map(|e| e == "retry").unwrap_or(false);
            async move { should_retry }
        }
    );

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "filter should allow retries until non-retriable error");
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn constant_backoff_delays_between_retries() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(2).unwrap())
        .backoff(ConstantBackoffStrategy::new(Duration::from_millis(100)))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_millis(100) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    tokio::time::sleep(Duration::from_millis(100) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn exponential_backoff_delays_grow() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(3).unwrap())
        .backoff(ExponentialBackoffStrategy::new(2.0))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_secs(1) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    tokio::time::sleep(Duration::from_secs(2) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    tokio::time::sleep(Duration::from_secs(4) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 4);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn exponential_backoff_clamped_at_max() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(3).unwrap())
        .backoff(ExponentialBackoffStrategy::new_with(3.0, Duration::from_secs(5)))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_secs(1) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    tokio::time::sleep(Duration::from_secs(3) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    tokio::time::sleep(Duration::from_secs(4)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    tokio::time::sleep(Duration::from_secs(1) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 4);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn linear_backoff_delays_grow() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(3).unwrap())
        .backoff(LinearBackoffStrategy::builder()
            .factor(Duration::from_secs(2))
            .start(Duration::from_secs(1))
            .build())
        .build();

    let handle = backoff_spawn!(frame);

    // retry=0 delay is 0s, so first two attempts happen back-to-back
    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    tokio::time::sleep(Duration::from_secs(2) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    tokio::time::sleep(Duration::from_secs(4) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 4);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn linear_backoff_clamped_at_max() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(3).unwrap())
        .backoff(LinearBackoffStrategy::builder()
            .factor(Duration::from_secs(2))
            .start(Duration::from_secs(1))
            .clamp(Duration::from_secs(3))
            .build())
        .build();

    let handle = backoff_spawn!(frame);

    // retry=0 delay is 0s, so first two attempts happen back-to-back
    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    tokio::time::sleep(Duration::from_secs(2) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    tokio::time::sleep(Duration::from_secs(3) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 4);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn jitter_full_delay_within_max() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(1).unwrap())
        .backoff(JitterBackoffStrategy::new_full(
            ConstantBackoffStrategy::new(Duration::from_secs(4)),
            1.0,
        ))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_secs(4) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn jitter_equal_delay_within_range() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(1).unwrap())
        .backoff(JitterBackoffStrategy::new_equal(
            ConstantBackoffStrategy::new(Duration::from_secs(4)),
            1.0,
        ))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_millis(1999)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_millis(2001) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    assert!(handle.await.unwrap().is_err());
}

#[tokio::test]
async fn jitter_decorrelated_delay_within_max() {
    tokio::time::pause();
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(1).unwrap())
        .backoff(JitterBackoffStrategy::new_decorrelated(
            ConstantBackoffStrategy::new(Duration::from_secs(2)),
            1.0,
            10.0,
        ))
        .build();

    let handle = backoff_spawn!(frame);

    tokio::time::sleep(NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_secs(6) + NS).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    assert!(handle.await.unwrap().is_err());
}
