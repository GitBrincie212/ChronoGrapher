use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chronographer::{
    prelude::DynamicTaskFrame,
    task::{Task, TaskFrameContext, TaskScheduleImmediate},
};

#[tokio::test]
async fn frame_execution_returns_ok() {
    let frame = DynamicTaskFrame::new(move |_ctx: &TaskFrameContext, _args: &()| async move {
        Ok::<_, String>(())
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
            Ok::<_, String>(())
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
        Err("intentional failure".to_string())
    });
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_err(), "Dynamic task should fail");
}

#[tokio::test]
async fn frame_execution_error_contains_message() {
    let frame = DynamicTaskFrame::new(|_ctx: &TaskFrameContext, _args: &()| async move {
        Err("specific error content".to_string())
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
            Ok::<_, String>(())
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
