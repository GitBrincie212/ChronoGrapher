use chronographer::prelude::DynamicTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use chronographer::task::TimeoutTaskFrame;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::impl_counting_frame;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "timeout error")
    }
}

impl_counting_frame!(DummyError);

const TIGHT_DURATION: Duration = Duration::from_millis(50);
const LARGE_DURATION: Duration = Duration::from_secs(1);

#[tokio::test]
async fn task_finishing_before_timeout_returns_ok() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    };
    let frame = TimeoutTaskFrame::new(frame, LARGE_DURATION);
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok());
}

#[tokio::test]
async fn task_finishing_after_timeout_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args| {
        let counter_clone = counter.clone();
        async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(51)).await;
            Ok::<_, DummyError>(())
        }
    });
    let frame = TimeoutTaskFrame::new(frame, TIGHT_DURATION);
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err())
}

#[tokio::test]
async fn task_returning_error_before_timeout_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    };
    let frame = TimeoutTaskFrame::new(frame, LARGE_DURATION);
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(
        exec.is_err(),
        "Frame error before timeout should propagate as error"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Frame should have been called once before returning error"
    );
}

#[tokio::test]
async fn zero_duration_timeout_returns_error() {
    let frame = DynamicTaskFrame::new(|_ctx: &TaskFrameContext, _args| async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, DummyError>(())
    });
    let frame = TimeoutTaskFrame::new(frame, Duration::from_nanos(0));
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err(), "Zero-duration timeout should immediately time out");
}
