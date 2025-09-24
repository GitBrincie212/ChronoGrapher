use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;

/// Represents an **execution task frame** that directly hosts and executes a function. This task frame type
/// acts as a **leaf node** within the task frame hierarchy. Its primary role is to serve as the final
/// unit of execution in a task workflow, as it only encapsulates a single function / future to be
/// executed, no further tasks can be chained or derived from it
///
/// # Events
/// When it comes to events, [`ExecutionTaskFrame`] comes with the default set of events, as
/// there is nothing else to listen for / subscribe to
///
/// # Example
/// ```ignore
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::Task;
///
/// let task_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Hello from an execution task!");
///         Ok(())
///     }
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs(2), task_frame);
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct ExecutionTaskFrame<F: Send + Sync>(F);

impl<F, Fut> ExecutionTaskFrame<F>
where
    Fut: Future<Output = Result<(), TaskError>> + Send,
    F: Fn(Arc<TaskContext>) -> Fut + Send + Sync,
{
    pub fn new(func: F) -> Self {
        ExecutionTaskFrame(func)
    }
}

#[async_trait]
impl<F, Fut> TaskFrame for ExecutionTaskFrame<F>
where
    Fut: Future<Output = Result<(), TaskError>> + Send,
    F: Fn(Arc<TaskContext>) -> Fut + Send + Sync,
{
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        self.0(ctx).await
    }
}
