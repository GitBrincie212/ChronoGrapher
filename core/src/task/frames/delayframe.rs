use crate::{define_event, define_event_group};
use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::TaskHookEvent;
use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

define_event!(
    /// [`OnDelayStart`] is an implementation of [`TaskHookEvent`] (a system used closely with [`TaskHook`]).
    /// The concrete payload type of [`OnDelayStart`] is ``Duration``, indicating the delay time it
    /// will take
    ///
    /// # Constructor(s)
    /// When constructing a [`OnDelayStart`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnDelayStart::default`] or via simply pasting
    /// the struct name ([`OnDelayStart`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnDelayStart`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnDelayStart`] is triggered when the [`DelayTaskFrame`]'s delay countdown is about to start
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnDelayStart`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`DelayTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnDelayStart, Duration
);

define_event!(
    /// [`OnDelayEnd`] is an implementation of [`TaskHookEvent`] (a system used closely with [`TaskHook`]).
    /// The concrete payload type of [`OnDelayEnd`] is ``Duration``, indicating the delay time it took
    ///
    /// # Constructor(s)
    /// When constructing a [`OnDelayEnd`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnDelayEnd::default`] or via simply pasting
    /// the struct name ([`OnDelayEnd`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnDelayEnd`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnDelayEnd`] is triggered when the [`DelayTaskFrame`]'s delay countdown has ended
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnDelayEnd`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`DelayTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnDelayEnd, Duration
);

define_event_group!(
    /// [`DelayEvents`] is a marker trait, more specifically a [`TaskHookEvent`] group of
    /// [`TaskHookEvent`] (a system used closely with [`TaskHook`]). It contains the common payload
    /// type of ``Duration`` which is the duration of the delay
    ///
    /// # Supertrait(s)
    /// Since it is a [`TaskHookEvent`] group, it requires every descended to implement the [`TaskHookEvent`],
    /// and more specifically have the payload type ``Duration``
    ///
    /// # Trait Implementation(s)
    /// Currently, two [`TaskHookEvent`] implement the [`DelayEvents`] marker trait
    /// (event group). Those being [`OnDelayStart`] and [`OnDelayEnd`]
    ///
    /// # Object Safety
    /// [`DelayEvents`] is **NOT** object safe, due to the fact it implements the
    /// [`TaskHookEvent`] which itself is not object safe
    ///
    /// # See Also
    /// - [`OnDelayStart`]
    /// - [`OnDelayEnd`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    DelayEvents, Duration |
    OnDelayStart, OnDelayEnd
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
    frame: Arc<T>,
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
            frame: Arc::new(frame),
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
        ctx.subdivide(self.frame.clone()).await
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
