use crate::errors::ChronographerErrors;
use crate::task::{ArcTaskEvent, TaskError, TaskEvent, TaskEventEmitter, TaskFrame, TaskMetadata};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait FrameAccessorFunc: Send + Sync {
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> usize;
}

#[async_trait]
impl<FAF: FrameAccessorFunc + ?Sized> FrameAccessorFunc for Arc<FAF> {
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> usize {
        self.as_ref().execute(metadata).await
    }
}

#[async_trait]
impl<F, Fut> FrameAccessorFunc for F
where
    F: Fn(Arc<dyn TaskMetadata>) -> Fut + Send + Sync,
    Fut: Future<Output = usize> + Send,
{
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> usize {
        self(metadata).await
    }
}

/// Represents a **select task frame** which wraps multiple task frames and picks one task frame based
/// on an accessor function. This task frame type acts as a **composite node** within the task frame hierarchy,
/// facilitating a way to conditionally execute a task frame from a list of multiple. The results
/// from the selected frame are returned when executed
///
/// # Events
/// For events, [`SelectTaskFrame`] has only a single event, that being `on_select` which executes when
/// a task frame is successfully selected (no index out of bounds) and before the target task frame
/// executes
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::selectframe::SelectTaskFrame;
/// use chronographer_core::task::Task;
///
/// // Picks it on the first run
/// let primary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // Picks it on the second run
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // Picks it on the third run
/// let tertiary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Tertiary task frame fired...");
///         Err(())
///     }
/// );
///
/// let select_frame = SelectTaskFrame::new(
///     vec![
///         Arc::new(primary_frame),
///         Arc::new(secondary_frame),
///         Arc::new(tertiary_frame)
///     ],
///
///     // Simple example, runs always is above zero so we can safely subtract one off it
///     |metadata| (metadata.runs() - 1) as usize % 3
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(3.21), select_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct SelectTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    accessor: Arc<dyn FrameAccessorFunc>,
    pub on_select: ArcTaskEvent<Arc<dyn TaskFrame>>,
}

impl SelectTaskFrame {
    pub fn new(tasks: Vec<Arc<dyn TaskFrame>>, accessor: impl FrameAccessorFunc + 'static) -> Self {
        Self {
            tasks,
            accessor: Arc::new(accessor),
            on_select: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl TaskFrame for SelectTaskFrame {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        let idx = self.accessor.execute(metadata.clone()).await;
        if let Some(frame) = self.tasks.get(idx) {
            emitter
                .emit(metadata.clone(), self.on_select.clone(), frame.clone())
                .await;
            return frame.execute(metadata, emitter).await;
        }
        Err(Arc::new(ChronographerErrors::TaskIndexOutOfBounds(
            idx,
            "SelectTaskFrame".to_owned(),
            self.tasks.len(),
        )))
    }
}
