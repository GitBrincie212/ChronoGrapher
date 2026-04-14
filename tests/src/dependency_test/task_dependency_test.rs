use std::fmt::{Debug, Display, Formatter};
use async_trait::async_trait;
use chronographer::errors::TaskError;
use chronographer::prelude::*;
use chronographer::task::{TaskFrame, TaskFrameContext, TaskScheduleImmediate};
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
struct StubError;

impl Display for StubError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

struct SimpleTaskFrame {
    should_succeed: Arc<AtomicBool>,
}

#[async_trait]
impl TaskFrame for SimpleTaskFrame {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Box::new(StubError))
        }
    }
}

#[tokio::test]
async fn test_task_dependency() {
    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame {
        should_succeed: should_succeed.clone(),
    };

    let task = Box::leak(Box::new(Task::new(TaskScheduleImmediate, frame)));

    let dep: TaskDependency = TaskDependency::new(task).await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(
        !dep.is_resolved().await,
        "Task dependency should not be resolved initially"
    );

    let result: Option<Box<dyn TaskError>> = None;
    task.emit_hook_event::<OnTaskEnd>(&result.as_ref().map(|x| x.as_ref()))
        .await;

    assert!(
        dep.is_resolved().await,
        "Task dependency should be resolved after task succeeds"
    );

    dep.disable().await;
    assert!(
        !dep.is_enabled().await,
        "Task dependency should be disabled when calling disable()"
    );

    dep.enable().await;
    assert!(
        dep.is_enabled().await,
        "Task dependency should be disabled when calling enable()"
    );

    dep.unresolve().await;
    assert!(
        !dep.is_resolved().await,
        "Task dependency should be unresolved when calling unresolve()"
    );

    dep.resolve().await;
    assert!(
        dep.is_resolved().await,
        "Task dependency should be resolved when calling resolve()"
    );
}

#[tokio::test]
async fn test_task_dependency_failure_only() {
    let should_succeed = Arc::new(AtomicBool::new(false));
    let frame = SimpleTaskFrame {
        should_succeed: should_succeed.clone(),
    };

    let task = Box::leak(Box::new(Task::new(TaskScheduleImmediate, frame)));

    let dep: TaskDependency = TaskDependency::builder(task)
        .resolve_behavior(TaskResolveFailureOnly)
        .minimum_runs(NonZeroU64::new(1).unwrap())
        .build()
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let success_result: Option<Box<dyn TaskError>> = None;
    task.emit_hook_event::<OnTaskEnd>(&success_result.as_ref().map(|x| x.as_ref()))
        .await;

    assert!(
        !dep.is_resolved().await,
        "Task dependency should not be resolved after task succeeds (failure only)"
    );

    let err_result: Option<Box<dyn TaskError>> =
        Some(Box::new(StubError));
    task.emit_hook_event::<OnTaskEnd>(&err_result.as_ref().map(|x| x.as_ref()))
        .await;

    assert!(
        dep.is_resolved().await,
        "Task dependency should be resolved after task fails"
    );
}
