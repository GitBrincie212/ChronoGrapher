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

define_event!(
    /// # Event Triggering
    /// [`OnFallbackEvent`] is triggered when the [`FallbackTaskFrame`]'s wrapped
    /// primary [`TaskFrame`] fails and switches to the wrapped secondary / fallback [`TaskFrame`]
    ///
    /// # See Also
    /// - [`FallbackTaskFrame`]
    OnFallbackEvent, (Arc<dyn TaskFrame>, TaskError)
);

/// Represents a **fallback task frame** which wraps two other task frames. This task frame type acts as a
/// **composite node** within the task frame hierarchy, providing a failover mechanism for execution.
///
/// # Constructor(s)
/// When constructing a [`FallbackTaskFrame`], the only way is via [`FallbackTaskFrame::new`]
/// which requires the two [`TaskFrame`], one primary and one fallback to construct
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
/// # Trait Implementation(s)
/// It is obvious that the [`FallbackTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
///
/// # Example
/// ```ignore
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::{FallbackTaskFrame, Task};
///
/// let primary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Trying primary task frame...");
///         Err::<(), ()>(())
///     }
/// );
///
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
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
pub struct FallbackTaskFrame<T: 'static, T2: 'static>(T, Arc<T2>);

impl<T, T2> FallbackTaskFrame<T, T2>
where
    T: TaskFrame + 'static,
    T2: TaskFrame + 'static,
{
    pub const PERSISTENCE_ID: &'static str = stringify!(FallbackTaskFrame$chronographer_core);

    /// Creates / Constructs a new [`FallbackTaskFrame`] instance based on the
    /// two [`TaskFrame`] supplied
    ///
    /// # Argument(s)
    /// The method accepts two arguments, those being ``primary`` which is a [`TaskFrame`]
    /// type and is the first task frame that will always execute. And the second being ``secondary``
    /// which is a [`TaskFrame`] type that is executed as last report option when the ``primary``
    /// fails
    ///
    /// # Returns
    /// A fully created [`FallbackTaskFrame`] with the primary
    /// task frame and a fallback task frame
    ///
    /// # See Also
    /// - [`ExecutionTaskFrame`]
    pub fn new(primary: T, secondary: T2) -> Self {
        Self(primary, Arc::new(secondary))
    }
}

#[async_trait]
impl<T, T2> TaskFrame for FallbackTaskFrame<T, T2>
where
    T: TaskFrame + 'static,
    T2: TaskFrame + 'static,
{
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let primary_result = self.0.execute(ctx.clone()).await;
        match primary_result {
            Err(err) => {
                ctx.clone()
                    .emit::<OnFallbackEvent>(&(self.1.clone(), err))
                    .await;
                self.1.execute(ctx).await
            }
            res => res,
        }
    }
}

#[async_trait]
impl<T, T2> PersistentObject for FallbackTaskFrame<T, T2>
where
    T: TaskFrame + 'static + PersistentObject,
    T2: TaskFrame + 'static + PersistentObject,
{
    fn persistence_id() -> &'static str {
        Self::PERSISTENCE_ID
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let primary = PersistenceUtils::serialize_persistent(&self.0).await?;
        let fallback = PersistenceUtils::serialize_persistent(self.1.as_ref()).await?;
        Ok(SerializedComponent::new::<Self>(json!({
            "primary_frame": primary,
            "fallback_frame": fallback,
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;

        let primary_frame = PersistenceUtils::deserialize_concrete::<T>(
            &mut repr,
            "primary_frame",
            "Cannot deserialize the primary wrapped task frame",
        )
        .await?;

        let fallback_frame = PersistenceUtils::deserialize_concrete::<T2>(
            &mut repr,
            "fallback_frame",
            "Cannot deserialize the primary wrapped task frame",
        )
        .await?;

        Ok(FallbackTaskFrame::new(primary_frame, fallback_frame))
    }
}
