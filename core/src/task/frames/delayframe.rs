use crate::define_event;
use crate::persistence::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

define_event!(
    /// # See Also
    /// - [`DelayTaskFrame`]
    OnDelayStart, Duration
);

define_event!(
    /// # See Also
    /// - [`DelayTaskFrame`]
    OnDelayEnd, Duration
);

/// Represents a **delay task frame** which wraps a [`TaskFrame`]. This task frame type acts as a
/// **wrapper node** within the [`TaskFrame`] hierarchy, providing a delay mechanism for execution.
///
/// # Constructor(s)
/// When constructing a [`DelayTaskFrame`], the only way to do it is via [`DelayTaskFrame::new`]
/// which accepts a [`TaskFrame`] along with a delay
///
/// # Events
/// [`DelayTaskFrame`] defines two events, and those are [`OnDelayStart`] and
/// [`OnDelayEnd`], the former is triggered when the delay starts while the
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
pub struct DelayTaskFrame<T: 'static>(T, Duration);

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
        DelayTaskFrame(frame, delay)
    }
}

#[async_trait]
impl<T: TaskFrame + 'static> TaskFrame for DelayTaskFrame<T> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        ctx.clone().emit::<OnDelayStart>(&self.1).await;
        tokio::time::sleep(self.1).await;
        ctx.clone().emit::<OnDelayEnd>(&self.1).await;
        self.0.execute(ctx).await
    }
}

#[async_trait]
impl<T: TaskFrame + PersistentObject> PersistentObject for DelayTaskFrame<T> {
    fn persistence_id() -> &'static str {
        "DelayTaskFrame$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let frame = PersistenceUtils::serialize_persistent(&self.0).await?;
        let delay = PersistenceUtils::serialize_field(self.1)?;
        Ok(SerializedComponent::new::<Self>(json!({
            "wrapped_frame": frame,
            "delay": delay
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;

        let delay = PersistenceUtils::deserialize_atomic::<Duration>(
            &mut repr,
            "delay",
            "Cannot deserialize the delay",
        )?;

        let frame = PersistenceUtils::deserialize_concrete::<T>(
            &mut repr,
            "wrapped_frame",
            "Cannot deserialize the wrapped task frame",
        )
        .await?;

        Ok(DelayTaskFrame::new(frame, delay))
    }
}
