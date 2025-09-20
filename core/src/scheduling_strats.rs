use crate::task::{Task, TaskErrorHandler, TaskEventEmitter};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;

/// [`ScheduleStrategy`] defines how the task behaves when being overlapped by the same instance
/// task or by others. Strategies receive a task, an event emitter and return a local timestamp
/// indicating the last time it was executed. It is their duty for strategies to correctly
/// call the lifecycle events, handle the run count and call the error handler (all of this is
/// automatically in [`<dyn ScheduleStrategy>::execute_logic`] for convenience’s sake
#[async_trait]
pub trait ScheduleStrategy: Send + Sync {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>);
}

#[async_trait]
impl<S: ScheduleStrategy + ?Sized> ScheduleStrategy for Arc<S> {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>) {
        self.as_ref().handle(task, emitter).await;
    }
}

/// [`SequentialSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the
/// task sequentially, only once it finishes, does it reschedule the same task. This is the default
/// scheduling strategy used by [`Task`]
pub struct SequentialSchedulingPolicy;
#[async_trait]
impl ScheduleStrategy for SequentialSchedulingPolicy {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>) {
        task.run(emitter).await.ok();
    }
}

/// [`ConcurrentSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the task
/// in the background, while it also reschedules other tasks to execute, one should be careful when
/// using this to not run into the [Thundering Herd Problem](https://en.wikipedia.org/wiki/Thundering_herd_problem)
pub struct ConcurrentSchedulingPolicy;
#[async_trait]
impl ScheduleStrategy for ConcurrentSchedulingPolicy {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>) {
        tokio::spawn(async move {
            task.run(emitter).await.ok();
        });
    }
}

/// [`CancelPreviousSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the
/// task in the background, unlike [`ConcurrentSchedulingPolicy`], this policy cancels the previous
/// task process if a new task overlaps it
///
/// **⚠ Note ⚠** due to a limitation, if the task frame executes CPU-Bound logic mostly and does not yield,
/// then the task frame may be completed, as such ensure the task frame has defined a sufficient
/// number of cancellation points / yields
pub struct CancelPreviousSchedulingPolicy(ArcSwapOption<JoinHandle<()>>);

impl Default for CancelPreviousSchedulingPolicy {
    fn default() -> Self {
        Self(ArcSwapOption::new(None))
    }
}

impl CancelPreviousSchedulingPolicy {
    pub fn new() -> Self {
        Self(ArcSwapOption::new(None))
    }
}

#[async_trait]
impl ScheduleStrategy for CancelPreviousSchedulingPolicy {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>) {
        let old_handle = self.0.swap(None);

        if let Some(handle) = old_handle {
            handle.abort();
        }

        let handle = tokio::spawn(async move {
            task.run(emitter).await.ok();
        });

        self.0.store(Some(Arc::new(handle)));
    }
}

/// [`CancelCurrentSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the
/// task in the background, unlike [`ConcurrentSchedulingPolicy`], this policy cancels the current
/// task that tries to overlaps the already-running task
pub struct CancelCurrentSchedulingPolicy(Arc<AtomicBool>);

impl Default for CancelCurrentSchedulingPolicy {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl CancelCurrentSchedulingPolicy {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

#[async_trait]
impl ScheduleStrategy for CancelCurrentSchedulingPolicy {
    async fn handle(&self, task: Arc<Task>, emitter: Arc<TaskEventEmitter>) {
        let is_free = &self.0;
        if !is_free.load(Ordering::Relaxed) {
            return;
        }
        is_free.store(false, Ordering::Relaxed);
        let is_free_clone = is_free.clone();
        tokio::spawn(async move {
            task.run(emitter).await.ok();
            is_free_clone.store(true, Ordering::Relaxed);
        });
    }
}
