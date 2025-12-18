use crate::define_event;
use crate::errors::ChronographerErrors;
use crate::task::{TaskContext, TaskError, TaskFrame, TaskHookEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// [`SelectFrameAccessor`] is a trait for selecting a task frame
/// along a list of potential candidates and is tied to [`SelectTaskFrame`]
///
/// # Required Method(s)
/// When implementing the [`SelectFrameAccessor`] trait, it is required to also
/// implement the method [`SelectFrameAccessor::select`] which is where the logic
/// for the actual selection takes place, it accepts a [`TaskContext`] wrapped in an ``Arc<T>``
/// and returns an index pointing to the task frame to execute
///
/// # Trait Implementation(s)
/// By default, [`SelectFrameAccessor`] trait is implemented on functions, however, due to their nature
/// of not being easily persistable, it is advised to implement the trait yourself
///
/// # Object Safety
/// [`SelectFrameAccessor`] is object safe as shown throughout [`SelectTaskFrame`]'s code
///
/// # See Also
/// - [`SelectTaskFrame`]
/// - [`TaskContext`]
#[async_trait]
pub trait SelectFrameAccessor: Send + Sync {
    /// The main logic for selecting a task frame to execute
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``ctx`` which is a
    /// [`TaskContext`] wrapped in an ``Arc<T>``
    ///
    /// # Returns
    /// An index that is type of ``usize`` pointing to the task frame. In case
    /// this is invalid [`SelectTaskFrame`] throws an error
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`SelectTaskFrame`]
    /// - [`SelectFrameAccessor`]
    async fn select(&self, ctx: &TaskContext) -> usize;
}

#[async_trait]
impl<FAF: SelectFrameAccessor + ?Sized> SelectFrameAccessor for Arc<FAF> {
    async fn select(&self, ctx: &TaskContext) -> usize {
        self.as_ref().select(ctx).await
    }
}

#[async_trait]
impl<F, Fut> SelectFrameAccessor for F
where
    F: Fn(&TaskContext) -> Fut + Send + Sync,
    Fut: Future<Output = usize> + Send,
{
    async fn select(&self, ctx: &TaskContext) -> usize {
        self(ctx).await
    }
}

define_event!(
    /// [`OnTaskFrameSelection`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnTaskFrameSelection`]
    /// is ``(usize, Arc<dyn TaskFrame>)``, the first value describes the index the [`TaskFrame`]
    /// was selected and the second value is the actual [`TaskFrame`]
    ///
    /// # Constructor(s)
    /// When constructing a [`OnTaskFrameSelection`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnTaskFrameSelection::default`] or via simply pasting
    /// the struct name ([`OnTaskFrameSelection`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnTaskFrameSelection`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnTaskFrameSelection`] is triggered when the [`SelectTaskFrame`] has
    /// successfully selected a [`TaskFrame`] (without an index out of bounds error)
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnTaskFrameSelection`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`SelectTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnTaskFrameSelection, (usize, Arc<dyn TaskFrame>)
);

/// Represents a **select task frame** which wraps multiple [`TaskFrame`] and picks one [`TaskFrame`] based
/// on an [`SelectFrameAccessor`]. This task frame type acts as a **composite node** within the [`TaskFrame`]
/// hierarchy, facilitating a way to conditionally execute a [`TaskFrame`] from a list of multiple.
/// The results from the selected frame are returned when executed
///
/// # Behavior
/// - When [`SelectTaskFrame`], it runs [`SelectFrameAccessor`]
/// - Based on the results of [`SelectFrameAccessor`], [`SelectTaskFrame`] determines if the index
///   is out of bounds, if it is, return an error otherwise proceed
/// - Emits the ``on_select`` event and executes the corresponding [`TaskFrame`]
///
/// # Constructor(s)
/// When constructing a [`SelectTaskFrame`], the only way to do so is via [`SelectTaskFrame::new`]
/// where you supply a collection of [`TaskFrame`] along with a [`SelectFrameAccessor`]
///
/// # Events
/// For events, [`SelectTaskFrame`] has only a single event, that being `on_select` which executes when
/// a task frame is successfully selected (no index out of bounds) and before the target task frame
/// executes
///
/// # Trait Implementation(s)
/// It is obvious that the [`SelectTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::selectframe::SelectTaskFrame;
/// use chronographer_core::task::Task;
///
/// // Picks it on the first run
/// let primary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // Picks it on the second run
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // Picks it on the third run
/// let tertiary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Tertiary task frame fired...");
///         Err(())
///     }
/// );
///
/// let select_frame = SelectTaskFrame::new(
///     vec![
///         Arc::new(primary_frame),
///         Arc::new(secondary_frame),
///         Arc::new(tertiary_frame)
///     ],
///
///     // Simple example, runs always is above zero so we can safely subtract one off it
///     |ctx| (ctx.runs() - 1) as usize % 3
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(3.21), select_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct SelectTaskFrame {
    frames: Vec<Arc<dyn TaskFrame>>,
    accessor: Arc<dyn SelectFrameAccessor>,
}

impl SelectTaskFrame {
    /// Creates / Constructs a new [`SelectTaskFrame`] instance
    ///
    /// # Argument(s)
    /// This method requires 2 arguments, those being a collection of [`TaskFrame`]
    /// as ``frames`` and a [`SelectFrameAccessor`] implementation as ``accessor``
    ///
    /// # Returns
    /// The fully constructed [`SelectTaskFrame`] with the collection of frames to
    /// select being ``frames`` and a [`SelectFrameAccessor`] being ``accessor``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`SelectFrameAccessor`]
    /// - [`SelectTaskFrame`]
    pub fn new(
        frames: Vec<Arc<dyn TaskFrame>>,
        accessor: impl SelectFrameAccessor + 'static,
    ) -> Self {
        Self {
            frames,
            accessor: Arc::new(accessor),
        }
    }
}

#[async_trait]
impl TaskFrame for SelectTaskFrame {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        let idx = self.accessor.select(ctx).await;
        if let Some(frame) = self.frames.get(idx) {
            ctx.emit::<OnTaskFrameSelection>(&(idx, frame.clone())).await;
            return ctx.subdivide(frame.clone()).await;
        }

        Err(Arc::new(ChronographerErrors::TaskIndexOutOfBounds(
            idx,
            "SelectTaskFrame".to_owned(),
            self.frames.len(),
        )))
    }
}
