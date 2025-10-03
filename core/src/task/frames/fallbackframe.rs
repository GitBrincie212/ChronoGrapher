use crate::deserialization_err;
use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::Debug;
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use crate::{acquire_mut_ir_map, deserialize_field, to_json};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

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
pub struct FallbackTaskFrame<T: 'static, T2: 'static> {
    primary: T,
    secondary: Arc<T2>,

    /// An event fired when the fallback is executed
    /// (i.e. The primary task frame failed)
    pub on_fallback: ArcTaskEvent<(Arc<T2>, TaskError)>,
}

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
        Self {
            primary,
            secondary: Arc::new(secondary),
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
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let primary_result = self.primary.execute(ctx.clone()).await;
        match primary_result {
            Err(err) => {
                ctx.emitter
                    .emit(
                        ctx.as_restricted(),
                        self.on_fallback.clone(),
                        (self.secondary.clone(), err),
                    )
                    .await;

                self.secondary.execute(ctx).await
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
    
    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        let primary = to_json!(self.primary.store().await?);
        let fallback = to_json!(self.secondary.store().await?);
        Ok(SerializedComponent::new::<Self>(
            json!({
                "primary_frame": primary,
                "fallback_frame": fallback,
            }),
        ))
    }

    async fn retrieve(
        component: SerializedComponent,
    ) -> Result<Self, TaskError> {
        let mut repr = acquire_mut_ir_map!(FallbackTaskFrame, component);

        deserialize_field!(
            repr,
            serialized_primary,
            "primary_frame",
            FallbackTaskFrame,
            "Cannot deserialize the primary wrapped task frame"
        );

        deserialize_field!(
            repr,
            serialized_fallback,
            "fallback_frame",
            FallbackTaskFrame,
            "Cannot deserialize the fallback wrapped task frame"
        );

        let primary_frame = T::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_primary.clone())
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await?;

        let fallback_frame = T2::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_fallback.clone())
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await?;

        Ok(FallbackTaskFrame::new(primary_frame, fallback_frame))
    }
}
