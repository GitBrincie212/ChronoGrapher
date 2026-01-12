pub mod default; // skipcq: RS-D1001

pub use default::*;
use std::any::Any;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerConfig;
use crate::task::{ErasedTask, TaskError};
use async_trait::async_trait;
use std::sync::Arc;

pub struct EngineNotifier {
    id: Box<dyn Any + Send + Sync>,
    notify: tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Option<TaskError>)>,
}

impl EngineNotifier {
    pub fn new<C: SchedulerConfig>(
        id: C::TaskIdentifier,
        notify: tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Option<TaskError>)>,
    ) -> Self {
        Self {
            id: Box::new(id),
            notify,
        }
    }

    pub async fn notify(self, result: Option<TaskError>) {
        self.notify
            .send((self.id, result))
            .await
            .expect("Failed to send notification via SchedulerTaskDispatcher, could not receive from the SchedulerEngine");
    }
}

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
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

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
    async fn dispatch(&self, task: Arc<ErasedTask>, rescheduler_notifier: EngineNotifier);
}
