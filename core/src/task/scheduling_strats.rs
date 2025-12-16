use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::{ErasedTask, TaskError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// [`ScheduleStrategy`] defines how the task should be rescheduled and how the task acts when being
/// overlapped by the same task instance or by others. It is their duty to handle calling
/// [`Task::run`] in their own way
///
/// # Required Methods
/// When implementing [`ScheduleStrategy`], one must supply an implementation for the method
/// [`ScheduleStrategy::handle`], which is where it handles the logic for running the task
///
/// # Trait Implementation(s)
/// There are multiple implementations to choose from, each fit for their own use-case. The core
/// provides 4 of these:
///
/// 1. [`SequentialSchedulingPolicy`] The default go-to option, the scheduler executes the [`Task`]
/// and waits for it to finish before rescheduling the same instance to re-run in the future
///
/// 2. [`ConcurrentSchedulingPolicy`] The scheduler executes the [`Task`] and immediately reschedules
/// it to re-run in the future. Careful handling must be present to prevent the thundering herd problem
/// (can be viewed more about it in the documentation of [`ConcurrentSchedulingPolicy`])
///
/// 3. [`CancelPreviousSchedulingPolicy`] Acts identical to the [`ConcurrentSchedulingPolicy`] but
/// instead of making it possible to overlap one or more similar instances of the task, when an overlap
/// happens, it cancels the previous and runs the current
/// 4. [`CancelCurrentSchedulingPolicy`] Acts identical to the [`ConcurrentSchedulingPolicy`] but
/// instead of making it possible to overlap one or more similar instances of the task, when an overlap
/// happens, it cancels the current and the previous continues running
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`Task`] struct
///
/// # See Also
/// - [`ScheduleStrategy`]
/// - [`Task`]
/// - [`TaskEventEmitter`]
/// - [`SequentialSchedulingPolicy`]
/// - [`ConcurrentSchedulingPolicy`]
/// - [`CancelPreviousSchedulingPolicy`]
/// - [`CancelCurrentSchedulingPolicy`]
#[async_trait]
pub trait ScheduleStrategy: 'static + Send + Sync {
    /// Runs the task from this strategy (which handles how the task should run)
    ///
    /// # Arguments
    /// - **task** The task instance to run via this strategy
    /// - **emitter** The event emitter to be used as argument for running the task
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskEventEmitter`]
    /// - [`ScheduleStrategy`]
    async fn handle(&self, task: Arc<ErasedTask>);
}

#[async_trait]
impl<S> ScheduleStrategy for S
where
    S: Deref + Send + Sync + 'static,
    S::Target: ScheduleStrategy,
{
    async fn handle(&self, task: Arc<ErasedTask>) {
        self.deref().handle(task).await;
    }
}

/// [`SequentialSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] which executes the
/// task sequentially, only once it finishes, does it reschedule the same task, ensuring no task
/// of the same instance may overlap. This is the default scheduling strategy used by [`Task`]
///
/// # Constructor(s)
/// One can simply just use the default rust struct initialization or use it with [`Default`]
///
/// # Trait Implementation(s)
/// [`SequentialSchedulingPolicy`] implements [`ScheduleStrategy`], as discussed above, as well as:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`ScheduleStrategy`]
/// - [`SequentialSchedulingPolicy::default`]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SequentialSchedulingPolicy;

#[async_trait]
impl ScheduleStrategy for SequentialSchedulingPolicy {
    async fn handle(&self, task: Arc<ErasedTask>) {
        let result: Result<(), TaskError> = task.run().await;
        result.ok();
    }
}

#[async_trait]
impl PersistenceObject for SequentialSchedulingPolicy {
    const PERSISTENCE_ID: &'static str =
        "chronographer::SequentialSchedulingPolicy#1bd3c9dc-46fd-4a83-b234-f575352e7e15";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}

/// [`ConcurrentSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the task
/// in the background, while it also reschedules other tasks to execute, one should be careful when
/// using this to not run into the [Thundering Herd Problem](https://en.wikipedia.org/wiki/Thundering_herd_problem)
///
/// # Constructor(s)
/// One can simply just use the default rust struct initialization or use it with [`Default`]
///
/// # Trait Implementation(s)
/// [`ConcurrentSchedulingPolicy`] implements [`ScheduleStrategy`], as discussed above, as well as:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`ScheduleStrategy`]
/// - [`ConcurrentSchedulingPolicy::default`]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ConcurrentSchedulingPolicy;

#[async_trait]
impl ScheduleStrategy for ConcurrentSchedulingPolicy {
    async fn handle(&self, task: Arc<ErasedTask>) {
        let cloned_task = task.clone();
        tokio::spawn(async move {
            let result: Result<(), TaskError> = cloned_task.run().await;
            result.ok();
        });
    }
}

#[async_trait]
impl PersistenceObject for ConcurrentSchedulingPolicy {
    const PERSISTENCE_ID: &'static str =
        "chronographer::ConcurrentSchedulingPolicy#06bae7de-0633-46bb-9c6c-169ad95b3eb1";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}

/// [`CancelPreviousSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the
/// task in the background, unlike [`ConcurrentSchedulingPolicy`], this policy cancels the previous
/// task process if a new task overlaps it
///
/// # Usage Note(s)
/// Due to a limitation, if the task frame executes CPU-Bound logic mostly and does not yield,
/// then the task frame may be completed. As such, ensure the task frame has defined a sufficient
/// number of cancellation points / yields
///
/// # Constructor(s)
/// One can simply use [`CancelPreviousSchedulingPolicy::default`] or
/// [`CancelPreviousSchedulingPolicy::new`] which act the same and are
/// mostly aliases
///
/// # Trait Implementation(s)
/// [`CancelPreviousSchedulingPolicy`] implements [`ScheduleStrategy`], as discussed above, as well as:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`ScheduleStrategy`]
/// - [`CancelPreviousSchedulingPolicy::default`]
pub struct CancelPreviousSchedulingPolicy(Mutex<Option<JoinHandle<()>>>);

impl Default for CancelPreviousSchedulingPolicy {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

impl Debug for CancelPreviousSchedulingPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancelPreviousSchedulingPolicy").finish()
    }
}

#[async_trait]
impl ScheduleStrategy for CancelPreviousSchedulingPolicy {
    async fn handle(&self, task: Arc<ErasedTask>) {
        let old_handle = self.0.lock().await.take();

        if let Some(handle) = old_handle {
            handle.abort();
        }

        let cloned_task = task.clone();
        let curr_handle = tokio::spawn(async move {
            let result: Result<(), TaskError> = cloned_task.run().await;
            result.ok();
        });

        *self.0.lock().await = Some(curr_handle);
    }
}

/*
#[async_trait]
impl PersistenceObject for CancelPreviousSchedulingPolicy {
    const PERSISTENCE_ID: &'static str = "chronographer::CancelPreviousSchedulingPolicy#7c49ad2e-9122-4ad3-8245-df2d77c1d464";

    fn inject_context<T: PersistenceBackend>(&self, _ctx: &PersistenceContext<T>) {}
}
 */

/// [`CancelCurrentSchedulingPolicy`] is an implementation of [`ScheduleStrategy`] and executes the
/// task in the background, unlike [`ConcurrentSchedulingPolicy`], this policy cancels the current
/// task that tries to overlaps the already-running task
///
/// # Constructor(s)
/// One can simply use [`CancelCurrentSchedulingPolicy::default`] or
/// [`CancelCurrentSchedulingPolicy::new`] which act the same and are
/// mostly aliases
///
/// # Trait Implementation(s)
/// [`CancelCurrentSchedulingPolicy`] implements [`ScheduleStrategy`], as discussed above, as well as:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Default`]
/// - [`Clone`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
#[derive(Clone, Serialize, Deserialize)]
pub struct CancelCurrentSchedulingPolicy(Arc<AtomicBool>);

impl Default for CancelCurrentSchedulingPolicy {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl Debug for CancelCurrentSchedulingPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancelPreviousSchedulingPolicy").finish()
    }
}

#[async_trait]
impl ScheduleStrategy for CancelCurrentSchedulingPolicy {
    async fn handle(&self, task: Arc<ErasedTask>) {
        let is_free = &self.0;
        if !is_free.load(Ordering::Relaxed) {
            return;
        }
        is_free.store(false, Ordering::Relaxed);
        let is_free_clone = is_free.clone();
        let cloned_task = task.clone();
        tokio::spawn(async move {
            let result: Result<(), TaskError> = cloned_task.run().await;
            result.ok();
            is_free_clone.store(true, Ordering::Relaxed);
        });
    }
}

#[async_trait]
impl PersistenceObject for CancelCurrentSchedulingPolicy {
    const PERSISTENCE_ID: &'static str =
        "chronographer::CancelCurrentSchedulingPolicy#dfce54d2-a4fb-478f-8b96-705ab4d3dba0";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}
