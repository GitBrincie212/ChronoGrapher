use crate::define_event;
use crate::persistence::PersistenceObject;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

define_event!(
    /// # Event Triggering
    /// [`OnTimeout`] is triggered when the [`TimeoutTaskFrame`] sees
    /// the wrapped [`TaskFrame`] has exceeded its maximum duration limit
    ///
    /// # See Also
    /// - [`TimeoutTaskFrame`]
    OnTimeout, ()
);

/// Represents a **timeout task frame** which wraps a [`TaskFrame`]. This task frame type acts as a
/// **wrapper node** within the [`TaskFrame`] hierarchy, providing a timeout mechanism for execution.
///
/// # Usage Note(s)
/// Due to a limitation, if the task frame executes CPU-Bound logic mostly and does not yield,
/// then the task frame may be completed. As such, ensure the wrapped [`TaskFrame`] has defined a sufficient
/// number of cancellation points / yields
///
/// # Constructor(s)
/// When constructing a [`TimeoutTaskFrame`], the only way to do it is via [`TimeoutTaskFrame::new`]
/// which accepts a [`TaskFrame`] along with a duration threshold to time out the task
///
/// # Behavior
/// - Executes the **wrapped task frame**.
/// - Tracks a timer while the task frame executes.
/// - If the task executes longer than a specified duration, an error
///   is thrown and the task is aborted
///
/// # Events
/// [`TimeoutTaskFrame`] defines one single event, and that is `on_timeout`, it executes when the
/// task frame is executing longer than the maximum duration allowed, it exposes no form of payload
///
/// # Trait Implementation(s)
/// It is obvious that the [`TimeoutTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however, it also implements
/// [`PersistenceObject`], [`Serialize`] and [`Deserialize`]. ONLY if the underlying
/// [`TaskFrame`] is persistable
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use tokio::time::sleep;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::timeoutframe::TimeoutTaskFrame;
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::Task;
///
/// let exec_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
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
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
///
/// # See Also
/// - [`TaskFrame`]
#[derive(Serialize, Deserialize)]
pub struct TimeoutTaskFrame<T: 'static> {
    frame: Arc<T>,
    max_duration: Duration,
} // TODO: Find a way to store the deadline of timeout

impl<T: TaskFrame + 'static> TimeoutTaskFrame<T> {
    /// Constructs / Creates a new [`TimeoutTaskFrame`] instance
    ///
    /// # Argument(s)
    /// The method accepts 2 arguments, those being ``frame`` as [`TaskFrame`] to wrap,
    /// and a maximum threshold duration as ``max_duration``
    ///
    /// # Returns
    /// A newly created [`TimeoutTaskFrame`] instance wrapping the [`TaskFrame`] as ``frame
    /// and having a maximum duration of ``max_duration``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`TimeoutTaskFrame`]
    pub fn new(frame: T, max_duration: Duration) -> Self {
        TimeoutTaskFrame {
            frame: Arc::new(frame),
            max_duration,
        }
    }
}

#[async_trait]
impl<T: TaskFrame + 'static> TaskFrame for TimeoutTaskFrame<T> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let result = tokio::time::timeout(self.max_duration, ctx.subdivide_exec(self.frame)).await;

        if let Ok(inner) = result {
            return inner;
        }

        ctx.emit::<OnTimeout>(&()).await; // skipcq: RS-E1015
        Err(Arc::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Task timed out",
        )))
    }
}

#[async_trait]
impl<T: TaskFrame + PersistenceObject> PersistenceObject for TimeoutTaskFrame<T> {
    const PERSISTENCE_ID: &'static str =
        "chronographer::TimeoutTaskFrame#cfbcfb94-5370-4b72-af3d-ceee31f7cea3";
}
