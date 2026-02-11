use crate::define_event;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, DynArcError, TaskFrame};
use async_trait::async_trait;
use std::sync::Arc;

define_event!(
    /// [`OnFallbackEvent`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnFallbackEvent`]
    /// is ``TaskError`` which is the same error the inner primary TaskFrame returned
    ///
    /// # Constructor(s)
    /// When constructing a [`OnFallbackEvent`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnFallbackEvent::default`] or via simply pasting
    /// the struct name ([`OnFallbackEvent`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnFallbackEvent`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnFallbackEvent`] is triggered when the [`FallbackTaskFrame`]'s wrapped
    /// primary [`TaskFrame`] fails and switches to the wrapped secondary / fallback [`TaskFrame`]
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnFallbackEvent`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`FallbackTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnFallbackEvent, DynArcError
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
/// is a part of the default provided implementations, however, it also implements
/// [`PersistenceObject`], [`Serialize`] and [`Deserialize`]. ONLY if the underlying
/// [`TaskFrame`] instances are persistable
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
pub struct FallbackTaskFrame<T, T2>(Arc<T>, Arc<T2>);

impl<T: TaskFrame, T2: TaskFrame> FallbackTaskFrame<T, T2> {
    /// Creates / Constructs a new [`FallbackTaskFrame`] instance based on the
    /// two [`TaskFrame`] supplied. This constructor is for TaskFrames which are not persistable
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
        Self(Arc::new(primary), Arc::new(secondary))
    }
}

#[async_trait]
impl<T: TaskFrame, T2: TaskFrame> TaskFrame for FallbackTaskFrame<T, T2> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), DynArcError> {
        match ctx.subdivide(self.0.clone()).await {
            Err(primary_error) => {
                ctx.emit::<OnFallbackEvent>(&primary_error).await;
                ctx.subdivide(self.1.clone()).await
            }
            res => res,
        }
    }
}
