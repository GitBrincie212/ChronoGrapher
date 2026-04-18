use chronographer::prelude::DynamicTaskFrame;
use chronographer::task::ConditionalFrame;
use chronographer::task::RestrictTaskFrameContext;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::impl_counting_frame;

#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "condition error")
    }
}

impl_counting_frame!(DummyError);

#[tokio::test]
async fn truthy_condition_returns_ok() {
    let counter = Arc::new(AtomicUsize::new(0));

    let counter_clone = counter.clone();
    let frame = DynamicTaskFrame::new(move |_ctx, _args: &()| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Ok::<_, DummyError>(())
        }
    });

    let predicate = |_ctx: &RestrictTaskFrameContext| async move { true };

    let frame = ConditionalFrame::builder()
        .frame(frame)
        .predicate(predicate)
        .build();

    let frame = Arc::new(frame);
    let frame = DynamicTaskFrame::new(move |ctx, _args: &()| {
        let ctx = *ctx;
        let frame = frame.clone();
        async move { frame.execute(&ctx, &((), ())).await }
    });

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased().run().await.unwrap();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn falsey_condition_runs_fallback() {
    let counter = Arc::new(AtomicUsize::new(0));
    let fallback_counter = Arc::new(AtomicUsize::new(0));

    let frame = CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    };

    let fallback = CountingFrame {
        counter: fallback_counter.clone(),
        should_fail: false,
    };

    let predicate = |_ctx: &RestrictTaskFrameContext| async move { false };

    let frame = ConditionalFrame::fallback_builder()
        .frame(frame)
        .fallback(fallback)
        .predicate(predicate)
        .build();

    let frame = Arc::new(frame);
    let frame = DynamicTaskFrame::new(move |ctx, _args: &()| {
        let ctx = *ctx;
        let frame = frame.clone();
        async move { frame.execute(&ctx, &((), ())).await }
    });

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased().run().await.unwrap();

    assert_eq!(counter.load(Ordering::SeqCst), 0);
    assert_eq!(fallback_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn falsey_condition_with_error_on_false_returns_error() {
    let frame = CountingFrame {
        counter: Arc::new(AtomicUsize::new(0)),
        should_fail: false,
    };

    let predicate = |_ctx: &RestrictTaskFrameContext| async move { false };

    let frame = ConditionalFrame::builder()
        .frame(frame)
        .predicate(predicate)
        .error_on_false(true)
        .build();

    let frame = Arc::new(frame);
    let frame = DynamicTaskFrame::new(move |ctx, _args: &()| {
        let ctx = *ctx;
        let frame = frame.clone();
        async move { frame.execute(&ctx, &((), ())).await }
    });

    let task = Task::new(TaskScheduleImmediate, frame);
    let result = task.into_erased().run().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn truthy_condition_with_failing_inner_frame_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let inner = CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    };

    let predicate = |_ctx: &RestrictTaskFrameContext| async move { true };

    let frame = ConditionalFrame::builder()
        .frame(inner)
        .predicate(predicate)
        .build();

    let frame = Arc::new(frame);
    let frame = DynamicTaskFrame::new(move |ctx, _args: &()| {
        let ctx = *ctx;
        let frame = frame.clone();
        async move { frame.execute(&ctx, &((), ())).await }
    });

    let task = Task::new(TaskScheduleImmediate, frame);
    let result = task.into_erased().run().await;

    assert!(result.is_err(), "Error from inner frame should propagate");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Inner frame should have been called once before failing"
    );
}

#[tokio::test]
async fn falsey_condition_with_failing_fallback_returns_error() {
    let primary_counter = Arc::new(AtomicUsize::new(0));
    let fallback_counter = Arc::new(AtomicUsize::new(0));

    let primary = CountingFrame {
        counter: primary_counter.clone(),
        should_fail: false,
    };

    let fallback = CountingFrame {
        counter: fallback_counter.clone(),
        should_fail: true,
    };

    let predicate = |_ctx: &RestrictTaskFrameContext| async move { false };

    let frame = ConditionalFrame::fallback_builder()
        .frame(primary)
        .fallback(fallback)
        .predicate(predicate)
        .build();

    let frame = Arc::new(frame);
    let frame = DynamicTaskFrame::new(move |ctx, _args: &()| {
        let ctx = *ctx;
        let frame = frame.clone();
        async move { frame.execute(&ctx, &((), ())).await }
    });

    let task = Task::new(TaskScheduleImmediate, frame);
    let result = task.into_erased().run().await;

    assert!(result.is_err(), "Error from failing fallback should propagate");
    assert_eq!(primary_counter.load(Ordering::SeqCst), 0, "Primary should not have run");
    assert_eq!(fallback_counter.load(Ordering::SeqCst), 1, "Fallback should have been called once");
}
