use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::utils::RescheduleAlerter;

/// [`DefaultTaskDispatcher`] is an implementation of [`SchedulerTaskDispatcher`],
/// this system works closely with the [`Scheduler`]. Its main job is to handle the execution of
/// more than one [`Task`] instances in such a way that its efficient and priority-based (lower
/// priority tasks execute commonly have time drifts on heavy workflow compared to critical tasks).
///
/// Due to the fact this system works with [`Scheduler`] it isn't really meant to be used outside
/// of this domain, rather it is just a composite of a much granular system
///
/// # Implementation Detail(s)
/// To achieve this, [`DefaultTaskDispatcher`] uses a thread pool via ``multipool`` under the hood,
/// configured for work-stealing and priority execution. Depending on the priority of a [`Task`],
/// [`DefaultTaskDispatcher`] ensures to execute at the precise timing, no matter what, delaying tasks
/// with lower priorities.
///
/// # Constructor(s)
/// When constructing a [`DefaultTaskDispatcher`], one can use either [`DefaultTaskDispatcher::new`]
/// to configure the worker / thread count, or [`DefaultTaskDispatcher::default`] via [`Default`]
/// trait
///
/// # Trait Implementation(s)
/// It is obvious that [`DefaultTaskDispatcher`] implements the [`SchedulerTaskDispatcher`], but
/// also the [`Default`] trait and [`Debug`] trait (although there are no fields when debugging it)
///
/// # See Also
/// - [`Scheduler`]
/// - [`Task`]
/// - [`SchedulerTaskDispatcher`]
#[derive(Default, Clone, Copy)]
pub struct DefaultTaskDispatcher;

#[async_trait]
impl<F: SchedulerConfig> SchedulerTaskDispatcher<F> for DefaultTaskDispatcher {
    async fn dispatch(
        &self,
        task: Arc<ErasedTask>,
        sender: &dyn RescheduleAlerter,
    ) {
        task.schedule_strategy().handle(task.clone()).await;
        sender.notify_task_finish().await;
    }
}
