use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use crate::{acquire_mut_ir_map, deserialization_err, deserialize_field, to_json};
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

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

    /// Event fired when a timeout occurs (i.e. The [`TaskFrame`] takes longer)
    pub on_timeout: ArcTaskEvent<()>,
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
            on_timeout: TaskEvent::new(),
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
        ctx.emitter
            .emit(ctx.as_restricted(), self.on_timeout.clone(), ())
            .await;
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
    
    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        let frame = to_json!(self.frame.store().await?);
        let max_duration = to_json!(self.max_duration);
        Ok(SerializedComponent::new::<Self>(
            json!({
                "wrapped_frame": frame,
                "max_duration": max_duration
            }),
        ))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = acquire_mut_ir_map!(TimeoutTaskFrame, component);

        deserialize_field!(
            repr,
            serialized_max_duration,
            "max_duration",
            TimeoutTaskFrame,
            "Cannot deserialize the maximum delay"
        );

        deserialize_field!(
            repr,
            serialized_frame,
            "wrapped_frame",
            TimeoutTaskFrame,
            "Cannot deserialize the wrapped task frame"
        );

        let delay: Duration = serde_json::from_value(serialized_max_duration).map_err(|_| {
            deserialization_err!(
                repr,
                TimeoutTaskFrame,
                "Cannot deserialize the maximum delay"
            )
        })?;

        let frame: T = T::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_frame.clone())
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await?;

        Ok(TimeoutTaskFrame::new(frame, delay))
    }
}
