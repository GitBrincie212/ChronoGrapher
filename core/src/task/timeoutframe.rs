use crate::task::{
    ArcTaskEvent, ExposedTaskMetadata, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter,
    TaskFrame, TaskStartEvent,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

/// Represents a **timeout task frame** which wraps a task frame. This task frame type acts as a
/// **wrapper node** within the task frame hierarchy, providing a timeout mechanism for execution.
///
/// # Behavior
/// - Executes the **wrapped task frame**.
/// - Tracks a timer while the task frame executes.
/// - If the task executes longer than a specified duration, an error
/// is thrown and the task is aborted
///
/// # ⚠ Important Note ⚠
/// Due to a limitation, if the task frame executes CPU-Bound logic mostly and does not yield,
/// then the task frame may be completed, as such ensure the task frame has defined a sufficient
/// number of cancellation points / yields
///
/// # Events
/// [`TimeoutTaskFrame`] defines one single event, and that is `on_timeout`, it executes when the
/// task frame is executing longer than the maximum duration allowed, it exposes no form of payload
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use tokio::time::sleep;
/// use chronolog_core::schedule::TaskScheduleInterval;
/// use chronolog_core::scheduler::{Scheduler, CHRONOLOG_SCHEDULER};
/// use chronolog_core::task::timeoutframe::TimeoutTaskFrame;
/// use chronolog_core::task::executionframe::ExecutionTaskFrame;
/// use chronolog_core::task::Task;
///
/// let exec_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Trying primary task...");
///         sleep(Duration::from_secs_f64(3.5)).await; // Suppose complex operations
///         Err::<(), ()>(())
///     }
/// );
///
/// let timeout_frame = TimeoutTaskFrame::new(
///     exec_frame,
///     Duration::from_secs(3)
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs(4), timeout_frame);
/// CHRONOLOG_SCHEDULER.register(task).await;
/// ```
pub struct TimeoutTaskFrame<T: 'static> {
    task: T,
    max_duration: Duration,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
    pub on_timeout: ArcTaskEvent<()>,
}

impl<T: TaskFrame + 'static> TimeoutTaskFrame<T> {
    pub fn new(task: T, max_duration: Duration) -> Self {
        TimeoutTaskFrame {
            task,
            max_duration,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            on_timeout: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl<T: TaskFrame + 'static> TaskFrame for TimeoutTaskFrame<T> {
    async fn execute(
        &self,
        metadata: Arc<dyn ExposedTaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        let result = tokio::time::timeout(
            self.max_duration,
            self.task.execute(metadata.clone(), emitter.clone()),
        )
        .await;

        if let Ok(inner) = result {
            return inner;
        }
        emitter
            .emit(metadata.clone(), self.on_timeout.clone(), ())
            .await;
        Err(Arc::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Task timed out",
        )))
    }

    fn on_start(&self) -> TaskStartEvent {
        self.on_start.clone()
    }

    fn on_end(&self) -> TaskEndEvent {
        self.on_end.clone()
    }
}
