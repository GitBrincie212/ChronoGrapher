use chronographer::task::DelayTaskFrame;
use chronographer::task::Task;
use chronographer::task::schedule::TaskScheduleImmediate;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::Instant;
use crate::task::frames::CountingFrame;

const TOLERANCE: Duration = Duration::from_millis(2);

const TIGHT: Duration = Duration::from_millis(15);
const LARGE: Duration = Duration::from_hours(24);

async fn run_delayed(delay: Duration) -> Result<Duration, String> {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = CountingFrame { counter, should_fail: false };
    let task_frame = DelayTaskFrame::new(frame, delay);
    let task = Task::new(TaskScheduleImmediate, task_frame);
    let start = Instant::now();
    let handle = tokio::spawn(async move {
        let exec = task.into_erased().run().await;
        assert!(exec.is_ok());
        exec
    });

    tokio::task::yield_now().await;
    tokio::time::advance(delay).await;

    handle.await.unwrap()?;
    let elapsed = start.elapsed();
    Ok(elapsed)
}

macro_rules! impl_delay_test {
    ($name: ident: $const: expr) => {
        #[tokio::test(start_paused = true)]
        async fn $name() -> Result<(), String> {
            let elapsed = run_delayed($const).await?;
            assert!(
                elapsed.abs_diff($const) < TOLERANCE,
                "The absolute difference of {elapsed:?} and {:?} is not close to the tolerance {TOLERANCE:?}",
                $const
            );

            Ok(())
        }
    };
}

impl_delay_test!(tight_delay_execute_as_expected: TIGHT);
impl_delay_test!(large_delay_execute_as_expected: LARGE);

#[tokio::test]
async fn task_execution_returns_ok() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter,
        should_fail: false,
    };

    let task_frame = DelayTaskFrame::new(frame, Duration::from_millis(1));
    let task = Task::new(TaskScheduleImmediate, task_frame);

    let exec = task.into_erased().run().await;
    assert!(exec.is_ok());
}

#[tokio::test(start_paused = true)]
async fn task_execution_with_delay_and_failing_frame_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    };

    let delay = Duration::from_secs(1);
    let task_frame = DelayTaskFrame::new(frame, delay);
    let task = Task::new(TaskScheduleImmediate, task_frame);

    let handle = tokio::spawn(async move {
        task.into_erased().run().await
    });

    tokio::task::yield_now().await;
    tokio::time::advance(delay).await;

    let exec = handle.await.unwrap();
    assert!(exec.is_err(), "Error from inner frame should propagate even after delay");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Inner frame should have been called once before failing"
    );
}

#[tokio::test(start_paused = true)]
async fn zero_duration_delay_executes_immediately() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    };

    let task_frame = DelayTaskFrame::new(frame, Duration::ZERO);
    let task = Task::new(TaskScheduleImmediate, task_frame);

    let handle = tokio::spawn(async move { task.into_erased().run().await });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::ZERO).await;

    let exec = handle.await.unwrap();
    assert!(exec.is_ok(), "Zero duration delay should still execute successfully");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
