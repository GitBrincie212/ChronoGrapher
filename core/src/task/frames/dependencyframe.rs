use crate::errors::ChronographerErrors;
use crate::task::dependency::FrameDependency;
use crate::task::{
    Arc, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter, TaskFrame, TaskMetadata,
    TaskStartEvent,
};
use async_trait::async_trait;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

/// [`DependentFailBehavior`] is a trait for implementing a behavior when dependencies aren't resolved
/// in [`DependencyTaskFrame`]. It takes nothing and returns a result for the [`DependencyTaskFrame`] to
/// return. By default, [`DependencyTaskFrame`] uses [`DependentSuccessOnFail`]
#[async_trait]
pub trait DependentFailBehavior: Send + Sync {
    async fn execute(&self) -> Result<(), TaskError>;
}

#[async_trait]
impl<DFB: DependentFailBehavior> DependentFailBehavior for Arc<DFB> {
    async fn execute(&self) -> Result<(), TaskError> {
        self.as_ref().execute().await
    }
}

/// When dependencies aren't resolved, return an error, more specifically
/// the ``ChronographerErrors::TaskDependenciesUnresolved`` error
pub struct DependentFailureOnFail;

#[async_trait]
impl DependentFailBehavior for DependentFailureOnFail {
    async fn execute(&self) -> Result<(), TaskError> {
        Err(Arc::new(ChronographerErrors::TaskDependenciesUnresolved))
    }
}

/// When dependencies aren't resolved, return a `Ok(())`
pub struct DependentSuccessOnFail;

#[async_trait]
impl DependentFailBehavior for DependentSuccessOnFail {
    async fn execute(&self) -> Result<(), TaskError> {
        Ok(())
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    task: T,
    dependencies: Vec<Arc<dyn FrameDependency>>,

    #[builder(
        default = Arc::new(DependentFailureOnFail),
        setter(transform = |ts: impl DependentFailBehavior + 'static| Arc::new(ts) as Arc<dyn DependentFailBehavior>)
    )]
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.task,
            dependencies: config.dependencies,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            dependent_behaviour: config.dependent_behaviour,
        }
    }
}

/// Represents an **dependent task frame** that directly wraps a task frame and executes it only if
/// all dependencies are resolved. This task frame type acts asa **wrapper node** within the task frame
/// hierarchy. Allowing the creation of task frames that depend on other tasks, in addition to allowing
/// dynamic execution (which opens the door for optimizations in case dependencies are expensive to compute)
///
/// # Events
/// When it comes to events, [`DependencyTaskFrame`] comes with the default set of events, as
/// there is nothing else to listen for / subscribe to
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::{DependencyTaskFrame, Task};
/// use chronographer_core::task::dependency::TaskDependency;
///
/// let exec_frame1 = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Hello from primary execution task!");
///         Ok(())
///     }
/// );
///
/// let exec_frame2 = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Hello from secondary execution task!");
///         Ok(())
///     }
/// );
///
/// let task1 = Arc::new(Task::define(TaskScheduleInterval::from_secs(5), exec_frame1));
/// let task1_dependency = TaskDependency::builder()
///     .task(task1.clone())
///     .build();
///
/// let dependent_frame2 = DependencyTaskFrame::builder()
///     .task(exec_frame2)
///     .dependencies(
///         vec![
///             Arc::new(task1_dependency)
///         ]
///     )
///     .build();
///
/// let task2 = Task::define(TaskScheduleInterval::from_secs(5), dependent_frame2);
///
/// CHRONOGRAPHER_SCHEDULER.schedule(task1.clone()).await;
/// ```
pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: T,
    dependencies: Vec<Arc<dyn FrameDependency>>,
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        let mut handles: Vec<JoinHandle<bool>> = Vec::new();

        for dep in &self.dependencies {
            let dep = dep.clone();
            handles.push(tokio::spawn(async move { dep.is_resolved().await }));
        }

        let mut is_resolved = true;
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if !result {
                        is_resolved = false;
                        break;
                    }
                }
                Err(_) => {
                    is_resolved = false;
                    break;
                }
            }
        }

        if !is_resolved {
            return self.dependent_behaviour.execute().await;
        }

        self.frame.execute(metadata.clone(), emitter).await
    }

    fn on_start(&self) -> TaskStartEvent {
        self.on_start.clone()
    }

    fn on_end(&self) -> TaskEndEvent {
        self.on_end.clone()
    }
}
