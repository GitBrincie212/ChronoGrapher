use crate::define_event;
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tokio::time::Instant;
use crate::persistence::{PersistenceContext, PersistenceObject};

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
#[derive(Serialize, Deserialize)]
pub struct DelayTaskFrame<T: TaskFrame> {
    frame: T,
    delay: Duration,
}

impl<T: TaskFrame> DelayTaskFrame<T> {
    /// Constructs / Creates a new [`DelayTaskFrame`] instance.
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
        }
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for DelayTaskFrame<T> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        ctx.emit::<OnDelayStart>(&self.delay).await;
        let deadline = Instant::now() + self.delay;
        tokio::time::sleep_until(deadline).await;
        ctx.emit::<OnDelayEnd>(&self.delay).await;
        ctx.subdivide(&self.frame).await
    }
}

#[async_trait]
impl<F: TaskFrame + PersistenceObject> PersistenceObject for DelayTaskFrame<F> {
    const PERSISTENCE_ID: &'static str =
        "chronographer::DelayTaskFrame#08656c89-041e-4b22-9c53-bb5a5e02a9f1";

    fn inject_context(&self, _ctx: &PersistenceContext) {
        todo!()
    }
}