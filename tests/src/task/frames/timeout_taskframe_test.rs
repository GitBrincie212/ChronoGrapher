use crate::task::frames::CountingFrame;
use chronographer::prelude::DynamicTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use chronographer::task::TimeoutTaskFrame;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

// TODO: Fix the errors in the unit tests regarding the timeouts

const TIGHT_DURATION: Duration = Duration::from_millis(50);
const LARGE_DURATION: Duration = Duration::from_secs(1);

#[tokio::test]
async fn task_finishing_before_timeout_returns_ok() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    };
    
    let frame = TimeoutTaskFrame::builder()
        .frame(frame)
        .duration(LARGE_DURATION)
        .build();

    let task = Task::new(frame, TaskScheduleImmediate);
    let exec = task.into_erased().run().await;

    assert!(
        exec.is_ok(),
        "Task should have succeeded without any errors"
    );
}

#[tokio::test]
async fn task_finishing_after_timeout_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args| {
        let counter_clone = counter.clone();
        async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = tokio::time::sleep(TIGHT_DURATION + Duration::from_millis(10)).await;
            Ok::<_, String>(())
        }
    });
    let frame = TimeoutTaskFrame::builder()
        .frame(frame)
        .duration(TIGHT_DURATION)
        .build();

    let task = Task::new(frame, TaskScheduleImmediate);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err(), "Task should have error-out with a timeout")
}

#[tokio::test]
async fn task_returning_error_before_timeout_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    };
    let frame = TimeoutTaskFrame::builder()
        .frame(frame)
        .duration(LARGE_DURATION)
        .build();

    let task = Task::new(frame, TaskScheduleImmediate);
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
        Ok::<_, String>(())
    });
    let frame = TimeoutTaskFrame::builder()
        .frame(frame)
        .duration(Duration::ZERO)
        .build();

    let task = Task::new(frame, TaskScheduleImmediate);
    let exec = task.into_erased().run().await;

    assert!(
        exec.is_err(),
        "Zero-duration timeout should immediately time out"
    );
}
