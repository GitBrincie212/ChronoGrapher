use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::{
    CollectionTaskFrame, GroupedTaskFramesQuitOnFailure, GroupedTaskFramesQuitOnSuccess,
    GroupedTaskFramesSilent, ParallelExecStrategy, SelectFrameAccessor, SelectionExecStrategy,
    SequentialExecStrategy, TaskFrame, TaskFrameContext, TaskScheduleImmediate,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct TestCollectionError(&'static str);

impl std::fmt::Display for TestCollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

struct CountingFrame {
    counter: Arc<AtomicUsize>,
    should_fail: bool,
}

#[async_trait]
impl TaskFrame for CountingFrame {
    type Error = TestCollectionError;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        if self.should_fail {
            Err(TestCollectionError("frame failed"))
        } else {
            Ok(())
        }
    }
}

struct FixedSelectAccessor(usize);

#[async_trait]
impl SelectFrameAccessor for FixedSelectAccessor {
    async fn select(&self, _ctx: &chronographer::task::RestrictTaskFrameContext<'_>) -> usize {
        self.0
    }
}

fn ok_frame(counter: &Arc<AtomicUsize>) -> Arc<dyn chronographer::task::ErasedTaskFrame> {
    Arc::new(CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    })
}

fn failing_frame(counter: &Arc<AtomicUsize>) -> Arc<dyn chronographer::task::ErasedTaskFrame> {
    Arc::new(CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    })
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
        .as_erased()
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
    task.as_erased()
        .run()
        .await
        .expect("silent should suppress failures");

    assert_eq!(counter.load(Ordering::SeqCst), 3);
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
    task.as_erased()
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
    task.as_erased()
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
        .as_erased()
        .run()
        .await
        .expect_err("selection should fail when index is out of bounds");

    assert_eq!(counter.load(Ordering::SeqCst), 0);
    assert_eq!(err.index(), 99);
}
