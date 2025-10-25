use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::{Task, TaskPriority};
use async_trait::async_trait;
use multipool::pool::ThreadPool;
use multipool::pool::modes::PriorityWorkStealingMode;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::broadcast;

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
pub struct DefaultTaskDispatcher {
    pool: ThreadPool<PriorityWorkStealingMode>,
}

impl Debug for DefaultTaskDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultTaskDispatcher").finish()
    }
}

impl Default for DefaultTaskDispatcher {
    fn default() -> Self {
        let pool = multipool::ThreadPoolBuilder::new()
            .set_work_stealing()
            .enable_priority()
            .num_threads(16)
            .build();
        Self { pool }
    }
}

impl DefaultTaskDispatcher {
    /// Creates / Constructs a [`DefaultTaskDispatcher`] instance
    ///
    /// # Argument(s)
    /// This method requests only one argument, that is the number of workers / threads via ``workers``
    /// to allocate for [`DefaultTaskDispatcher`]. The more workers / threads, the more CPU power
    /// it consumes but handles more tasks concurrently, minimizing major time drifts under heavy
    /// workflow
    ///
    /// # Return(s)
    /// The newly created [`DefaultTaskDispatcher`] instance with a configured number of
    /// workers / threads set to ``workers``
    ///
    /// # See Also
    /// - [`DefaultTaskDispatcher`]
    pub fn new(workers: usize) -> Self {
        let pool = multipool::ThreadPoolBuilder::new()
            .set_work_stealing()
            .enable_priority()
            .num_threads(workers)
            .build();
        Self { pool }
    }
}

#[async_trait]
impl SchedulerTaskDispatcher for DefaultTaskDispatcher {
    async fn dispatch(
        self: Arc<Self>,
        sender: Arc<broadcast::Sender<usize>>,
        task: Arc<Task>,
        idx: usize,
    ) {
        let target_priority = match task.priority() {
            TaskPriority::CRITICAL => 0,
            TaskPriority::IMPORTANT => 100,
            TaskPriority::HIGH => 200,
            TaskPriority::MODERATE => 300,
            TaskPriority::LOW => 400,
        };

        let idx_clone = idx;
        self.pool.spawn_with_priority(
            move || {
                let idx_clone = idx_clone;
                let task_clone = task.clone();
                let sender_clone = sender.clone();
                async move {
                    task_clone
                        .clone()
                        .schedule_strategy()
                        .handle(task_clone.clone())
                        .await;
                    sender_clone.send(idx_clone).unwrap();
                }
            },
            target_priority,
        );
    }
}
