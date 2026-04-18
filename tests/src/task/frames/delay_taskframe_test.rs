use crate::impl_counting_frame;
use chronographer::task::DelayTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::trigger::schedule::TaskScheduleImmediate;
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::Instant;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error")
    }
}

impl_counting_frame!(DummyError);

macro_rules! in_range_of {
    ($n:expr, ms) => {{
        let base = std::time::Duration::from_millis($n);
        let tol = std::time::Duration::from_millis(5);
        (base.saturating_sub(tol))..(base.saturating_add(tol))
    }};
    ($n:expr, s) => {{
        let base = std::time::Duration::from_secs($n);
        let tol = std::time::Duration::from_secs(1);
        (base.saturating_sub(tol))..(base.saturating_add(tol))
    }};
    ($n:expr, min) => {
        std::time::Duration::from_secs($n * 60)
    };
}

/// 15 milliseconds
const MS: Duration = Duration::from_millis(15);
/// 7 seconds
const S: Duration = Duration::from_secs(7);
/// 2 minutes
const MIN: Duration = Duration::from_secs(2 * 60);

macro_rules! impl_delay_test {
    (ms)  => { impl_delay_test!(@range ms, 15, MS); };
    (s)   => { impl_delay_test!(@range s, 7, S); };
    (min) => { impl_delay_test!(@exact min, 2, MIN); };

    (@range $unit:ident, $expected:literal, $const:ident) => {
        paste::paste! {
            #[tokio::test(start_paused = true)]
            async fn [<$unit _delay_execute_as_expected>]() {
                let counter = Arc::new(AtomicUsize::new(0));
                let frame = CountingFrame { counter, should_fail: false };
                let task_frame = DelayTaskFrame::new(frame, $const);
                let task = Task::new(TaskScheduleImmediate, task_frame);

                let start = Instant::now();
                let handle = tokio::spawn(async move {
                    let exec = task.into_erased().run().await;
                    assert!(exec.is_ok());
                    exec
                });

                tokio::task::yield_now().await;
                tokio::time::advance($const).await;

                handle.await.unwrap().unwrap();
                let elapsed = start.elapsed();

                let range = in_range_of!($expected, $unit);
                println!("{elapsed:?}");
                assert!(
                    range.contains(&elapsed),
                    "Execution took {:?}, expected between {:?} and {:?}",
                    elapsed,
                    range.start,
                    range.end
                );

            }
        }
    };

    (@exact $unit:ident, $expected:literal, $const:ident) => {
        paste::paste! {
            #[tokio::test(start_paused = true)]
            async fn [<$unit _delay_execute_as_expected>]() {
                let counter = Arc::new(AtomicUsize::new(0));
                let frame = CountingFrame { counter, should_fail: false };
                let task_frame = DelayTaskFrame::new(frame, $const);
                let task = Task::new(TaskScheduleImmediate, task_frame);

                let start = Instant::now();
                let handle = tokio::spawn(async move {
                    task.into_erased().run().await
                });

                tokio::task::yield_now().await;
                tokio::time::advance($const).await;

                handle.await.unwrap().unwrap();
                let elapsed = start.elapsed();

                let min_duration = in_range_of!($expected, $unit);
                assert!(
                    elapsed >= min_duration,
                    "Execution took {:?}, expected at least {:?}",
                    elapsed,
                    min_duration
                );
            }
        }
    };
}

impl_delay_test!(ms);
impl_delay_test!(s);
impl_delay_test!(min);

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

    let delay = Duration::from_millis(15);
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
