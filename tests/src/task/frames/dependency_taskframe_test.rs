use chronographer::prelude::DynamicDependency;
use chronographer::prelude::FrameDependency;
use chronographer::task::DependencyTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::impl_counting_frame;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dependency error")
    }
}

impl_counting_frame!(DummyError);

fn ok_deps(num: usize) -> Vec<Arc<dyn FrameDependency>> {
    (0..num)
        .map(|_| Arc::new(DynamicDependency::new(|| async { true })) as Arc<dyn FrameDependency>)
        .collect()
}

fn failing_deps(num: usize) -> Vec<Arc<dyn FrameDependency>> {
    (0..num)
        .map(|_| Arc::new(DynamicDependency::new(|| async { false })) as Arc<dyn FrameDependency>)
        .collect()
}

#[tokio::test]
async fn returns_ok_when_all_deps_resolved() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = DependencyTaskFrame::builder()
        .frame(CountingFrame {
            counter: counter.clone(),
            should_fail: false,
        })
        .dependencies(ok_deps(3))
        .build();
    let task = Task::new(TaskScheduleImmediate, frame);
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
        .dependencies(failing_deps(1))
        .build();
    let task = Task::new(TaskScheduleImmediate, frame);
    let result = task.into_erased().run().await;
    assert!(result.is_err());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "inner frame should not have been called"
    );
}
