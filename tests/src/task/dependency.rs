use std::num::NonZeroU16;
use chronographer::prelude::FrameDependency;
use chronographer::task::{Task, TaskScheduleImmediate};
use crate::task::utils::CountingTaskFrame;

#[tokio::test]
async fn test_basic_dependency() {
    let dep = FrameDependency::external(|| async { true });
    assert!(
        dep.is_resolved().await,
        "Dependency should be resolved to true based on its future"
    );
    assert!(
        !dep.is_disabled(),
        "Dependency should be enabled by default"
    );

    dep.disable();
    assert!(
        dep.is_disabled(),
        "Dependency should be disabled after calling disable()"
    );

    dep.enable();
    assert!(
        !dep.is_disabled(),
        "Dependency should be enabled after calling enable()"
    );
}

#[tokio::test]
async fn test_and_dependency() {
    let logical_dep1 = FrameDependency::external(|| async { true }) & FrameDependency::external(|| async { true });
    let logical_dep2 = FrameDependency::external(|| async { false }) & FrameDependency::external(|| async { true });
    let logical_dep3 = FrameDependency::external(|| async { true }) & FrameDependency::external(|| async { false });
    let logical_dep4 = FrameDependency::external(|| async { false }) & FrameDependency::external(|| async { false });

    assert!(
        logical_dep1.is_resolved().await,
        "Dependency should be resolved (T AND T = T)"
    );

    assert!(
        !logical_dep2.is_resolved().await,
        "Dependency should not be resolved (F AND T = F)"
    );

    assert!(
        !logical_dep3.is_resolved().await,
        "Dependency should not be resolved (T AND F = F)"
    );

    assert!(
        !logical_dep4.is_resolved().await,
        "Dependency should not be resolved (F AND F = F)"
    );
}

#[tokio::test]
async fn test_or_dependency() {
    let logical_dep1 = FrameDependency::external(|| async { true }) | FrameDependency::external(|| async { true });
    let logical_dep2 = FrameDependency::external(|| async { false }) | FrameDependency::external(|| async { true });
    let logical_dep3 = FrameDependency::external(|| async { true }) | FrameDependency::external(|| async { false });
    let logical_dep4 = FrameDependency::external(|| async { false }) | FrameDependency::external(|| async { false });

    assert!(
        logical_dep1.is_resolved().await,
        "Dependency should be resolved (T OR T = T)"
    );

    assert!(
        logical_dep2.is_resolved().await,
        "Dependency should be resolved (F OR T = T)"
    );

    assert!(
        logical_dep3.is_resolved().await,
        "Dependency should be resolved (T OR F = T)"
    );

    assert!(
        !logical_dep4.is_resolved().await,
        "Dependency should not be resolved (F OR F = F)"
    );
}

#[tokio::test]
async fn test_not_dependency() {
    let logical_dep1 = !FrameDependency::external(|| async { true });
    let logical_dep2 = !FrameDependency::external(|| async { false });

    assert!(
        !logical_dep1.is_resolved().await,
        "Dependency should not be resolved (NOT T = F)"
    );

    assert!(
        logical_dep2.is_resolved().await,
        "Dependency should be resolved (NOT F = T)"
    );
}

#[tokio::test]
async fn test_task_identity_run_dependency() -> Result<(), String> {
    let frame = CountingTaskFrame::default();
    let task = Task::new(frame.clone(), TaskScheduleImmediate);
    let dep1 = FrameDependency::runs(&task, NonZeroU16::MIN).await;
    let dep2 = FrameDependency::runs(&task, NonZeroU16::MAX).await;
    let dep3 = FrameDependency::runs(&task, NonZeroU16::MIN).await;
    let dep4 = FrameDependency::runs(&task, NonZeroU16::MAX).await;

    assert!(
        !dep1.is_resolved().await,
        "Task dependency with minimum run of one should not be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.enable_failure();
    let erased = task.into_erased();
    erased.run().await?;

    assert!(
        dep1.is_resolved().await,
        "Task dependency with minimum run of one should be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.disable_failure();
    erased.run().await?;

    assert!(
        dep1.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );
    
    erased.run().await?;

    assert!(
        dep3.is_resolved().await,
        "Task dependency with minimum run of one should be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.enable_failure();
    erased.run().await?;

    assert!(
        dep3.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    Ok(())
}

#[tokio::test]
async fn test_task_success_run_dependency() -> Result<(), String> {
    let frame = CountingTaskFrame::default();
    let task = Task::new(frame.clone(), TaskScheduleImmediate);
    let dep1 = FrameDependency::successful_runs(&task, NonZeroU16::MIN).await;
    let dep2 = FrameDependency::successful_runs(&task, NonZeroU16::MAX).await;
    let dep3 = FrameDependency::runs(&task, NonZeroU16::MIN).await;
    let dep4 = FrameDependency::runs(&task, NonZeroU16::MAX).await;

    assert!(
        !dep1.is_resolved().await,
        "Task dependency with minimum successful run of one should not be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum successful runs should not be resolved"
    );

    frame.enable_failure();
    let erased = task.into_erased();
    erased.run().await?;

    assert!(
        !dep1.is_resolved().await,
        "Task dependency with minimum successful run of one should not be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum successful runs should not be resolved"
    );

    frame.disable_failure();
    erased.run().await?;

    assert!(
        dep1.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    erased.run().await?;

    assert!(
        dep3.is_resolved().await,
        "Task dependency with minimum run of one should be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.enable_failure();
    erased.run().await?;

    assert!(
        dep3.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    Ok(())
}

#[tokio::test]
async fn test_task_failed_run_dependency() -> Result<(), String> {
    let frame = CountingTaskFrame::default();
    let task = Task::new(frame.clone(), TaskScheduleImmediate);
    let dep1 = FrameDependency::failed_runs(&task, NonZeroU16::MIN).await;
    let dep2 = FrameDependency::failed_runs(&task, NonZeroU16::MAX).await;
    let dep3 = FrameDependency::runs(&task, NonZeroU16::MIN).await;
    let dep4 = FrameDependency::runs(&task, NonZeroU16::MAX).await;

    assert!(
        !dep1.is_resolved().await,
        "Task dependency with minimum run of one should not be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.enable_failure();
    let erased = task.into_erased();
    erased.run().await?;

    assert!(
        dep1.is_resolved().await,
        "Task dependency with minimum run of one should be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.disable_failure();
    erased.run().await?;

    assert!(
        dep1.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep2.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );
    
    erased.run().await?;

    assert!(
        !dep3.is_resolved().await,
        "Task dependency with minimum run of one should not be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    frame.enable_failure();
    erased.run().await?;

    assert!(
        dep3.is_resolved().await,
        "Task dependency with minimum run of one should still be resolved"
    );

    assert!(
        !dep4.is_resolved().await,
        "Task dependency with maximum runs should not be resolved"
    );

    Ok(())
}