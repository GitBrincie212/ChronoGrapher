use crate::task::frames::CountingFrame;
use chronographer::prelude::FrameDependency;
use chronographer::task::DependencyTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskScheduleImmediate;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

fn ok_dependency() -> FrameDependency {
    FrameDependency::external(|| async { true })
}

fn failing_dependency() -> FrameDependency {
    FrameDependency::external(|| async { false })
}

#[tokio::test]
async fn returns_ok_when_all_deps_resolved() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DependencyTaskFrame::builder()
        .frame(CountingFrame {
            counter: counter.clone(),
            should_fail: false,
        })
        .dependency(ok_dependency() & ok_dependency() & ok_dependency())
        .build();
    let task = Task::new(frame, TaskScheduleImmediate);
    task.into_erased().run().await.unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn returns_error_when_dep_unresolved() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DependencyTaskFrame::builder()
        .frame(CountingFrame {
            counter: counter.clone(),
            should_fail: false,
        })
        .dependency(failing_dependency())
        .build();
    let task = Task::new(frame, TaskScheduleImmediate);
    let result = task.into_erased().run().await;
    assert!(result.is_err());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "inner frame should not have been called"
    );
}

#[tokio::test]
async fn stop_on_first_failing_dep() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DependencyTaskFrame::builder()
        .frame(CountingFrame {
            counter: counter.clone(),
            should_fail: false,
        })
        .dependency(ok_dependency() & ok_dependency() & failing_dependency())
        .build();
    let task = Task::new(frame, TaskScheduleImmediate);
    let result = task.into_erased().run().await;
    assert!(
        result.is_err(),
        "Should fail when at least one dependency is unresolved"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Inner frame should not have been called when deps fail"
    );
}

#[tokio::test]
async fn inner_frame_fails_when_all_deps_resolve() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DependencyTaskFrame::builder()
        .frame(CountingFrame {
            counter: counter.clone(),
            should_fail: true,
        })
        .dependency(ok_dependency() & ok_dependency())
        .build();
    let task = Task::new(frame, TaskScheduleImmediate);
    let result = task.into_erased().run().await;
    assert!(
        result.is_err(),
        "Should propagate inner frame error even when all deps resolve"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Inner frame should have been called and failed"
    );
}
