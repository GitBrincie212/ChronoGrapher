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
pub struct DummyError(&'static str);

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
