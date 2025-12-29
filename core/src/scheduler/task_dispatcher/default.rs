use crate::scheduler::RescheduleNotifier;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::ErasedTask;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;

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
#[derive(Default)]
pub struct DefaultTaskDispatcher;

impl Debug for DefaultTaskDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultTaskDispatcher").finish()
    }
}

#[async_trait]
impl SchedulerTaskDispatcher for DefaultTaskDispatcher {
    async fn dispatch<T: 'static + Send + Sync>(
        &self,
        task: Arc<ErasedTask>,
        sender: RescheduleNotifier<T>,
    ) {
        task.schedule_strategy().handle(task.clone()).await;
        sender
            .notify()
            .map_err(|_| "Dispatch finish message was not received successfully")
            .unwrap();
    }
}
