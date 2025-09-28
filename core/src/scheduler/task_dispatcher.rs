#[allow(missing_docs)]
pub mod default; // skipcq: RS-D1001

pub use default::*;
use std::fmt::Debug;

use crate::task::{Task, TaskEventEmitter};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;

/// [`SchedulerTaskDispatcher`] is a trait for implementing a scheduler task dispatcher. It acts as
/// a central point for when a task wants to execute, it does not handle scheduling, in fact it
/// communicates closely to the [`Scheduler`] to stay informed on whenever to execute the task
///
/// # Required Method(s)
/// When implementing the [`SchedulerTaskDispatcher`], one has to supply an implementation for
/// the method [`SchedulerTaskDispatcher::dispatch`] which is where the dispatching logic lives
///
/// # Trait Implementation(s)
/// There is one implementation in particular for [`SchedulerTaskDispatcher`], that is [`DefaultTaskDispatcher`]
/// which has a thread pool that handles work-stealing and priority management to ensure that
/// ChronoGrapher stays responsive even under heavy workflow
///
/// # Object Safety
/// [`SchedulerTaskDispatcher`] is object safe as seen throughout [`Scheduler`] source code
///
/// # See Also
/// - [`SchedulerTaskDispatcher::dispatch`]
/// - [`Scheduler`]
/// - [`DefaultTaskDispatcher`]
#[async_trait]
pub trait SchedulerTaskDispatcher: Debug + Send + Sync {
    /// The main logic of the [`SchedulerTaskDispatcher`]. This is where it handles
    /// how to execute a specified task and notify the [`Scheduler`] accordingly
    ///
    /// # Argument(s)
    /// The method accepts 4 arguments, those being a ``sender`` for notifying the
    /// [`Scheduler`] when the [`Task`] has finished executing, an ``emitter`` which is
    /// for event emission (the dispatcher by itself doesn't emit events. It is used
    /// as an argument for running a task), a [`Task`] represented as ``task`` and an
    /// index pointing to the [`Task`]
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`Scheduler`]
    /// - [`SchedulerTaskDispatcher`]
    async fn dispatch(
        self: Arc<Self>,
        sender: Arc<Sender<usize>>,
        emitter: Arc<TaskEventEmitter>,
        task: Arc<Task>,
        idx: usize,
    );
}

#[async_trait]
impl<TD: SchedulerTaskDispatcher + ?Sized> SchedulerTaskDispatcher for Arc<TD> {
    async fn dispatch(
        self: Arc<Self>,
        sender: Arc<Sender<usize>>,
        emitter: Arc<TaskEventEmitter>,
        task: Arc<Task>,
        idx: usize,
    ) {
        self.as_ref()
            .clone()
            .dispatch(sender, emitter, task, idx)
            .await
    }
}
