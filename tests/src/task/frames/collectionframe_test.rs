use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::{
    CollectionTaskFrame, GroupedTaskFramesQuitOnFailure, GroupedTaskFramesQuitOnSuccess,
    GroupedTaskFramesSilent, ParallelExecStrategy, SelectFrameAccessor, SelectionExecStrategy,
    SequentialExecStrategy, TaskFrame, TaskFrameContext, TaskScheduleImmediate,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::impl_counting_frame;

#[derive(Debug)]
struct TestCollectionError(&'static str);

impl std::fmt::Display for TestCollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl_counting_frame!(TestCollectionError);

struct FixedSelectAccessor(usize);

#[async_trait]
impl SelectFrameAccessor for FixedSelectAccessor {
    async fn select(&self, _ctx: &RestrictTaskFrameContext) -> usize {
        self.0
    }
}

#[tokio::test]
async fn sequential_quit_on_first_frame_returns_indexed_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            failing_frame(&counter),
            ok_frame(&counter),
            ok_frame(&counter),
        ],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );
    let task = Task::new(TaskScheduleImmediate, frame);
    let err = task
        .into_erased()
        .run()
        .await
        .expect_err("Sequential starategy should stop on failure");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(err.index(), 0);
}

#[tokio::test]
async fn sequential_quit_on_failure_returns_indexed_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            ok_frame(&counter),
            failing_frame(&counter),
            ok_frame(&counter),
        ],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    let err = task
        .into_erased()
        .run()
        .await
        .expect_err("sequential strategy should stop on failure");

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert_eq!(err.index(), 1);
}

#[tokio::test]
async fn sequential_silent_runs_all_frames() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            ok_frame(&counter),
            failing_frame(&counter),
            ok_frame(&counter),
        ],
        SequentialExecStrategy::new(GroupedTaskFramesSilent),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect("silent should suppress failures");

    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn sequential_fails_on_first_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            failing_frame(&counter),
            failing_frame(&counter),
            failing_frame(&counter),
        ],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    let err = task
        .into_erased()
        .run()
        .await
        .expect_err("Should fails on all tasks");

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(err.index(), 0);
}

#[tokio::test]
async fn parallel_quit_on_success_returns_early() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            ok_frame(&counter),
            failing_frame(&counter),
            failing_frame(&counter),
        ],
        ParallelExecStrategy::new(GroupedTaskFramesQuitOnSuccess),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect("parallel should return success once any frame succeeds");

    assert!(counter.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn selection_exec_runs_selected_frame_only() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter), ok_frame(&counter), ok_frame(&counter)],
        SelectionExecStrategy::new(FixedSelectAccessor(2)),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect("selection should succeed");

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn selection_exec_out_of_bounds_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter), ok_frame(&counter)],
        SelectionExecStrategy::new(FixedSelectAccessor(99)),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    let err = task
        .into_erased()
        .run()
        .await
        .expect_err("selection should fail when index is out of bounds");

    assert_eq!(counter.load(Ordering::SeqCst), 0);
    assert_eq!(err.index(), 99);
}

// NOTE: This test may be changed in the future since the behavior of an empty
// array of task frames may change
#[tokio::test]
async fn empty_tasks_exec_test() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame =
        CollectionTaskFrame::new(vec![], SelectionExecStrategy::new(FixedSelectAccessor(0)));

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect_err("Running no tasks should suceed");
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn parallel_all_fail_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            failing_frame(&counter),
            failing_frame(&counter),
            failing_frame(&counter),
        ],
        ParallelExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect_err("When all parallel frames fail with QuitOnFailure, the collection should return an error");

    assert!(counter.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn parallel_quit_on_failure_returns_error_on_first_fail() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![
            failing_frame(&counter),
            ok_frame(&counter),
            ok_frame(&counter),
        ],
        ParallelExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect_err("QuitOnFailure should return error when any frame fails");

    assert!(counter.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn selection_exec_selects_failing_frame_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter), failing_frame(&counter), ok_frame(&counter)],
        SelectionExecStrategy::new(FixedSelectAccessor(1)),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect_err("Selecting a failing frame should propagate the error");

    assert_eq!(counter.load(Ordering::SeqCst), 1, "Only the selected frame should have run");
}

#[tokio::test]
async fn empty_sequential_collection_returns_ok() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = CollectionTaskFrame::new(
        vec![],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let task = Task::new(TaskScheduleImmediate, frame);
    task.into_erased()
        .run()
        .await
        .expect("Empty sequential collection should succeed with no frames to run");

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}
