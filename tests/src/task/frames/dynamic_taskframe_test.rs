use chronographer::task::TaskFrame;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chronographer::{
    prelude::DynamicTaskFrame,
    task::{Task, TaskFrameContext, TaskScheduleImmediate},
};

use crate::impl_counting_frame;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error")
    }
}

impl_counting_frame!(DummyError);

#[tokio::test]
async fn frame_execution_returns_ok() {
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args: &()| async move {
        Ok::<_, DummyError>(())
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok(), "Dynamic task should succeed");
}

#[tokio::test]
async fn frame_execution_increments_counter() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args: &()| {
        let counter = counter_clone.clone();
        async move {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok::<_, DummyError>(())
        }
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok(), "Dynamic task should succeed");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn frame_execution_returns_error() {
    let frame = DynamicTaskFrame::new(|_ctx: &TaskFrameContext, _args: &()| async move {
        Err(DummyError("intentional failure"))
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err(), "Dynamic task should fail");
}

#[tokio::test]
async fn frame_execution_error_contains_message() {
    let frame = DynamicTaskFrame::new(|_ctx: &TaskFrameContext, _args: &()| async move {
        Err(DummyError("specific error content"))
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err(), "Dynamic task should fail");
    let err_msg = exec.unwrap_err().to_string();
    assert!(
        err_msg.contains("error"),
        "Error message should be propagated, got: {err_msg}"
    );
}

#[tokio::test]
async fn frame_closure_captures_state_across_multiple_runs() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args: &()| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Ok::<_, DummyError>(())
        }
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let erased = task.into_erased();

    erased.run().await.unwrap();
    erased.run().await.unwrap();
    erased.run().await.unwrap();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        3,
        "Counter should reflect all three executions"
    );
}
