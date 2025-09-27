use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Represents a **delay task frame** which wraps a [`TaskFrame`]. This task frame type acts as a
/// **wrapper node** within the [`TaskFrame`] hierarchy, providing a delay mechanism for execution.
///
/// # Constructor(s)
/// When constructing a [`DelayTaskFrame`], the only way to do it is via [`DelayTaskFrame::new`]
/// which accepts a [`TaskFrame`] along with a delay
///
/// # Events
/// [`DelayTaskFrame`] defines two events, and that is [`DelayTaskFrame::on_delay_start`] and 
/// [`DelayTaskFrame::on_delay_end`], the former is triggered when the delay starts while the 
/// latter is fired when the delay ends
///
/// # Trait Implementation(s)
/// It is obvious that the [`DelayTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use tokio::time::sleep;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::delayframe::DelayTaskFrame;
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::Task;
///
/// let exec_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Trying primary task...");
///         sleep(Duration::from_secs_f64(1.234)).await; // Suppose complex operations
///         Err::<(), ()>(())
///     }
/// );
///
/// let timeout_frame = DelayTaskFrame::new(
///     exec_frame,
///     Duration::from_secs(3)
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs(4), timeout_frame);
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
///
/// # See Also
/// - [`TaskFrame`]
pub struct DelayTaskFrame<T: 'static> {
    frame: T,
    delay: Duration,

    /// Event fired when the delay is triggered
    pub on_delay_start: ArcTaskEvent<Duration>,

    /// Event fired when the delay ends
    pub on_delay_end: ArcTaskEvent<Duration>,
}

impl<T: TaskFrame + 'static> DelayTaskFrame<T> {
    /// Constructs / Creates a new [`DelayTaskFrame`] instance
    ///
    /// # Argument(s)
    /// The method accepts 2 arguments, those being ``frame`` as [`TaskFrame`] to wrap,
    /// and a delay via ``delay``
    ///
    /// # Returns
    /// A newly created [`DelayTaskFrame`] instance wrapping the [`TaskFrame`] as ``frame
    /// and having a delay of ``delay``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`DelayTaskFrame`]
    pub fn new(frame: T, delay: Duration) -> Self {
        DelayTaskFrame {
            frame,
            delay,
            on_delay_start: TaskEvent::new(),
            on_delay_end: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl<T: TaskFrame + 'static> TaskFrame for DelayTaskFrame<T> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        ctx.emitter
            .emit(
                ctx.metadata.clone(),
                self.on_delay_start.clone(),
                self.delay.clone()
            )
            .await;
        tokio::time::sleep(self.delay).await;
        ctx.emitter
            .emit(
                ctx.metadata.clone(),
                self.on_delay_end.clone(),
                self.delay.clone()
            )
            .await;
        let result = self.frame.execute(ctx.clone()).await;
        result
    }
}
