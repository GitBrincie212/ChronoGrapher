use crate::task::{
    ArcTaskEvent, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter, TaskFrame, TaskMetadata,
    TaskStartEvent,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Represents a **fallback task frame** which wraps two other task frames. This task frame type acts as a
/// **composite node** within the task frame hierarchy, providing a failover mechanism for execution.
///
/// # Behavior
/// - Executes the **primary task frame** first.
/// - If the primary task frame completes successfully, the fallback task frame is **skipped**.
/// - If the primary task frame **fails**, the **secondary task frame** is executed as a fallback.
///
/// # Events
/// [`FallbackTaskFrame`] includes one event for when the fallback is triggered. Handing out the fallback
/// task frame instance being executed as well as the task error which can be accessed via the `on_fallback`
/// field
///
/// # Example
/// ```ignore
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::{FallbackTaskFrame, Task};
///
/// let primary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Trying primary task frame...");
///         Err::<(), ()>(())
///     }
/// );
///
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Primary failed, running fallback task frame!");
///         Ok::<(), ()>(())
///     }
/// );
///
/// let fallback_frame = FallbackTaskFrame::new(primary_frame, secondary_frame);
///
/// let task = Task::define(TaskScheduleInterval::from_secs(1), fallback_frame);
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct FallbackTaskFrame<T: 'static, T2: 'static> {
    primary: T,
    secondary: Arc<T2>,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
    pub on_fallback: ArcTaskEvent<(Arc<T2>, TaskError)>,
}

impl<T, T2> FallbackTaskFrame<T, T2>
where
    T: TaskFrame + 'static,
    T2: TaskFrame + 'static,
{
    pub fn new(primary: T, secondary: T2) -> Self {
        Self {
            primary,
            secondary: Arc::new(secondary),
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            on_fallback: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl<T, T2> TaskFrame for FallbackTaskFrame<T, T2>
where
    T: TaskFrame + 'static,
    T2: TaskFrame + 'static,
{
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        let primary_result = self
            .primary
            .execute(metadata.clone(), emitter.clone())
            .await;
        match primary_result {
            Err(err) => {
                emitter
                    .emit(
                        metadata.clone(),
                        self.on_fallback.clone(),
                        (self.secondary.clone(), err),
                    )
                    .await;

                self.secondary.execute(metadata, emitter).await
            }
            res => res,
        }
    }

    fn on_start(&self) -> TaskStartEvent {
        self.on_start.clone()
    }

    fn on_end(&self) -> TaskEndEvent {
        self.on_end.clone()
    }
}
