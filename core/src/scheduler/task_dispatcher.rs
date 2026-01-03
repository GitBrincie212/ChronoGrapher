pub mod default; // skipcq: RS-D1001

pub use default::*;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerConfig;
use crate::task::ErasedTask;
use crate::utils::RescheduleAlerter;
use async_trait::async_trait;
use std::sync::Arc;

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
pub trait SchedulerTaskDispatcher<F: SchedulerConfig>: 'static + Send + Sync {
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
    async fn dispatch(&self, task: Arc<ErasedTask>, rescheduler_notifier: &dyn RescheduleAlerter);
}
