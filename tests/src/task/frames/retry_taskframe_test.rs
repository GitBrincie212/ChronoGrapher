use chronographer::task::{RetriableTaskFrame, Task, TaskFrame, TaskFrameContext, TaskScheduleImmediate};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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

#[tokio::test]
async fn retry_succeeds_eventually() {
    let counter = Arc::new(AtomicUsize::new(0));
    let fail_times = 2;

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times })
        .retries(NonZeroU32::new(3).unwrap())
        .constant(Duration::ZERO)
        .build();

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

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(retries).unwrap())
        .constant(Duration::ZERO)
        .build();

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

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: 0 })
        .retries(NonZeroU32::new(3).unwrap())
        .constant(Duration::ZERO)
        .build();

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

    let frame = RetriableTaskFrame::builder()
        .frame(FailNTimesFrame { counter: counter.clone(), fail_times: usize::MAX })
        .retries(NonZeroU32::new(3).unwrap())
        .constant(Duration::ZERO)
        .when(|_err: Option<&String>| async { false })
        .build();

    let result = Task::new(frame, TaskScheduleImmediate).into_erased().run().await;

    assert!(result.is_ok(), "when=false stops retry and swallows error");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "should only attempt once before filter stops retry"
    );
}
