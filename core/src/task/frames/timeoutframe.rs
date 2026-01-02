use crate::define_event;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

define_event!(
    /// [`OnTimeout`] is an implementation of [`TaskHookEvent`] (a system used closely with [`TaskHook`]).
    /// The concrete payload type of [`OnTimeout`] is ``Duration`` indicating the maximum duration
    /// allowed for the [`TaskFrame`] to run
    ///
    /// # Constructor(s)
    /// When constructing a [`OnTimeout`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnTimeout::default`] or via simply pasting
    /// the struct name ([`OnTimeout`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnTimeout`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnTimeout`] is triggered when the [`TimeoutTaskFrame`] sees
    /// the wrapped [`TaskFrame`] has exceeded its maximum duration limit
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnTimeout`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`TimeoutTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnTimeout, Duration
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
pub struct TimeoutTaskFrame<T: TaskFrame> {
    frame: Arc<T>,
    max_duration: Duration,
}

impl<T: TaskFrame> TimeoutTaskFrame<T> {
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
        Self {
            frame: Arc::new(frame),
            max_duration,
        }
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        let result =
            tokio::time::timeout(self.max_duration, ctx.subdivide(self.frame.clone())).await;

        if let Ok(inner) = result {
            return inner;
        }

        ctx.emit::<OnTimeout>(&self.max_duration).await; // skipcq: RS-E1015
        Err(Arc::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Task timed out",
        )))
    }
}