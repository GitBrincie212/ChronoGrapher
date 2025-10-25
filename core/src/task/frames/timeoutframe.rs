use crate::define_event;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
/// is a part of the default provided implementations, however there are many others
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
pub struct TimeoutTaskFrame<T: 'static> {
    frame: T,
    max_duration: Duration,
}

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
            frame,
            max_duration,
        }
    }
}

#[async_trait]
impl<T: TaskFrame + 'static> TaskFrame for TimeoutTaskFrame<T> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let result = tokio::time::timeout(self.max_duration, self.frame.execute(ctx.clone())).await;

        if let Ok(inner) = result {
            return inner;
        }
        ctx.emit::<OnTimeout>(&()).await;
        Err(Arc::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Task timed out",
        )))
    }
}

#[async_trait]
impl<T: TaskFrame + PersistentObject> PersistentObject for TimeoutTaskFrame<T> {
    fn persistence_id() -> &'static str {
        "TimeoutTaskFrame$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let frame = PersistenceUtils::serialize_persistent(&self.frame).await?;
        let max_duration = PersistenceUtils::serialize_field(self.max_duration)?;
        Ok(SerializedComponent::new::<Self>(json!({
            "wrapped_frame": frame,
            "max_duration": max_duration
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;

        let delay = PersistenceUtils::deserialize_atomic::<Duration>(
            &mut repr,
            "max_duration",
            "Cannot deserialize the maximum delay",
        )?;

        let frame = PersistenceUtils::deserialize_concrete::<T>(
            &mut repr,
            "wrapped_frame",
            "Cannot deserialize the wrapped task frame",
        )
        .await?;

        Ok(TimeoutTaskFrame::new(frame, delay))
    }
}
