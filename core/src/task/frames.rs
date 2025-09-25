pub mod conditionframe;
pub mod dependencyframe;
pub mod executionframe;
pub mod fallbackframe;
pub mod noopframe;
pub mod parallelframe;
pub mod retryframe;
pub mod selectframe;
pub mod sequentialframe;
pub mod timeoutframe;

use std::fmt::Debug;
use crate::task::events::TaskEventEmitter;
use crate::task::{Task, TaskMetadata, TaskPriority};
use async_trait::async_trait;
pub use conditionframe::ConditionalFrame;
pub use dependencyframe::DependencyTaskFrame;
pub use executionframe::ExecutionTaskFrame;
pub use fallbackframe::FallbackTaskFrame;
pub use parallelframe::ParallelTaskFrame;
pub use retryframe::RetriableTaskFrame;
pub use selectframe::SelectTaskFrame;
pub use sequentialframe::SequentialTaskFrame;
use std::num::NonZeroU64;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;
pub use timeoutframe::TimeoutTaskFrame;

/// A task-related error (i.e. A task failure)
pub type TaskError = Arc<dyn Debug + Send + Sync>;

/// [`TaskContext`] is a mechanism that wraps commonly needed information
/// inside it which can be accessed by [`TaskFrame`], it essentially wraps
/// - metadata: [`TaskMetadata`]
/// - emitter: [`TaskEventEmitter`]
/// - priority: [`TaskPriority`]
/// - runs: ``u64``
/// - debug_label: ``String``
/// - max_runs: ``Option<NonZeroU64>``
///
/// All of them fetched in [`Task`]
///
/// # Constructor(s)
/// There are no public constructors as this context's constructor is sealed
///
/// # Task Implementation(s)
/// The [`TaskContext`] only implements [`Clone`] and [`Debug`] as there is no other use for it.
/// Where for [`Debug`] it outputs all the fields except the event emitter (it holds no data, so no
/// point in recording that)
///
/// # See Also
/// - [`Task`]
/// - [`TaskFrame`]
/// - [`TaskMetadata`]
/// - [`TaskPriority`]
/// - [`TaskEventEmitter`]
#[derive(Clone)]
pub struct TaskContext {
    pub(crate) metadata: Arc<TaskMetadata>,
    pub(crate) emitter: Arc<TaskEventEmitter>,
    pub(crate) priority: TaskPriority,
    pub(crate) runs: u64,
    pub(crate) debug_label: String,
    pub(crate) max_runs: Option<NonZeroU64>,
}

impl Debug for TaskContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskContext")
            .field("metadata", self.metadata.as_ref())
            .field("priority", &self.priority)
            .field("runs", &self.runs)
            .field("debug_label", &&self.debug_label)
            .field("max_runs", &self.max_runs)
            .finish()
    }
}

impl TaskContext {
    /// Constructs / Creates a new [`TaskContext`] instance based for use inside [`TaskFrame`],
    /// unlike most constructors, this mechanism is sealed and accessible only in the library's
    /// internal code
    ///
    /// # Argument(s)
    /// This method accepts 2 arguments, those being [`Task`] and [`TaskEventEmitter`] wrapped in an
    /// ``Arc<T>``, the former is for retrieving most of the data. While the latter is not only to prove
    /// that this construction is made internally on [`Task`] but also to share it with [`TaskFrame`]
    /// to emit its own events
    ///
    /// # Returns
    /// The constructed instance to be used
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskEventEmitter`]
    /// - [`TaskContext`]
    pub(crate) fn new(task: &Task, emitter: Arc<TaskEventEmitter>) -> Arc<Self> {
        Arc::new(Self {
            metadata: task.metadata.clone(),
            emitter,
            priority: task.priority,
            runs: task.runs.load(Ordering::Relaxed),
            debug_label: task.debug_label.clone(),
            max_runs: task.max_runs,
        })
    }

    /// Accesses the metadata field, returning it in the process
    ///
    /// # Returns
    /// The metadata field as an ``Arc<TaskMetadata>``
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskMetadata`]
    pub fn metadata(&self) -> Arc<TaskMetadata> {
        self.metadata.clone()
    }

    /// Accesses the event emitter field, returning it in the process
    ///
    /// # Returns
    /// The event emitter field as an ``Arc<TaskEventEmitter>``
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskEventEmitter`]
    pub fn emitter(&self) -> Arc<TaskEventEmitter> {
        self.emitter.clone()
    }

    /// Accesses the priority field, returning it in the process
    ///
    /// # Returns
    /// The priority field as an [`TaskPriority`]
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskPriority`]
    pub fn priority(&self) -> TaskPriority {
        self.priority.clone()
    }

    /// Accesses the runs field (counts how many times the task ran), returning it in the process
    ///
    /// # Returns
    /// The runs field as a typical ``u64``
    ///
    /// # See Also
    /// - [`TaskContext`]
    pub fn runs(&self) -> u64 {
        self.runs
    }

    /// Accesses the debug label field, returning it in the process
    ///
    /// # Returns
    /// The debug label field as a typical ``&str``
    ///
    /// # See Also
    /// - [`TaskContext`]
    pub fn debug_label(&self) -> &str {
        self.debug_label.as_str()
    }

    /// Accesses the max_runs field, returning it in the process
    ///
    /// # Returns
    /// The max_runs field as a ``Option<NonZeroU64>``
    ///
    /// # See Also
    /// - [`TaskContext`]
    pub fn max_runs(&self) -> Option<NonZeroU64> {
        self.max_runs
    }
}

/// [`TaskFrame`] represents a unit of work which hosts the actual computation logic for
/// the [`Scheduler`] to invoke, this is a part of the task system
///
/// # Required Method(s)
/// When one implements the [`TaskFrame`] trait, one has to implement [`TaskFrame::execute`] which
/// encapsulates mainly the async execution logic of the [`Task`], the method has as argument
/// a [`TaskContext`], wrapping any essential logic (while also being impossible to create it
/// outside the [`Task`]), the method also returns either `Ok(())` on success, or a `TaskError`
/// on failure. The method also handles the emission of local task frame events
/// (learn more about in [`TaskEvent`])
///
/// # Usage Notes
/// This is one of many components which are combined to form a task, other components are needed
/// to fuse them to a task. By itself it is not as useful, as such combine it to form a task with other
/// composites
///
/// [`TaskFrame`] can be decorated with other task unit implementations to expand the behavior, such
/// as adding retry mechanism via [`RetryFrame`], adding timeout via [`TimeoutFrame`]... etc. Some
/// examples (from simple to complex) include:
/// - **RetriableTaskFrame<T>** Executes a task frame ``T``, when the task frame succeeds at some point
/// it stops and returns the success results. Otherwise, if it fails, it retires it ``N`` times (controlled
/// by the developer) til it succeeds, or it reaches this threshold with a specified backoff retry strategy
///
/// - **RetriableTaskFrame<DependencyTaskFrame<T>>** Executes a task frame ``T``, if all of its
/// dependencies are resolved and ``T`` succeeds at some point (both have to be satisfied), it stops
/// and returns the success results. Otherwise, if it fails, it retires it ``N`` (controlled by the
/// developer) til it succeeds with a specified backoff retry strategy
///
/// - **``RetriableTaskFrame<TimeoutTaskFrame<T>>``**: Execute task frame `T`, when the
/// task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
/// then finish, otherwise if it exceeds its maximum duration or if the task frame failed then
/// abort it and retry it again, repeating this process `N` times (can be controlled by the developer)
/// with a specified backoff retry strategy
///
/// - **``FallbackTaskFrame<TimeoutTaskFrame<T1>, RetriableTaskFrame<T2>>``**: Execute task frame `T1`,
/// when the task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
/// then finish, otherwise if it either fails or it reaches its maximum duration then execute
/// task frame `T2` (as a fallback), try/retry executing this task frame for `N` times (can be
/// controlled by the developer) with a delay per retry of `d` (can be controlled by the developer),
/// regardless if it succeeds at some time or fails entirely, return the result back
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`Task`] struct
///
/// # Trait Implementation(s)
/// There are various implementations for [`TaskFrame`] present in the library, each
/// doing their own part. Some noteworthy mentions include
/// - [`RetriableTaskFrame`] Retries a task frame a specified number of times with a delay
/// per retry based on a supplied [`RetryBackoffStrategy`] (more info on the docs of it)
///
/// - [`TimeoutTaskFrame`] Runs a task frame for a specified duration, if the countdown reaches
/// zero, then it halts the task and returns a timeout error (more info on the docs of it)
///
/// - [`DependencyTaskFrame`] Before running a task frame, it checks if its dependencies are resolved,
/// if they are then it runs, otherwise it errors out with dependencies unresolved
///
/// - [`FallbackTaskFrame`] Attempts to run a task frame, if it fails, then a fallback secondary task frame
/// takes its place which the result of this fallback task frame is returned from the secondary (more
/// info on the docs of it)
///
/// - [`NoOperationTaskFrame`] Effectively acts as a placeholder and does nothing useful, in
/// some circumstances where task frames may be optional to supply (more info on the docs of it)
///
/// It is advised to check the submodules of the task module to see more of them in action
///
/// # See Also
/// - [`TaskFrame::execute`]
/// - [`RetriableTaskFrame`]
/// - [`TimeoutTaskFrame`]
/// - [`DependencyTaskFrame`]
/// - [`FallbackTaskFrame`]
/// - [`NoOperationTaskFrame`]
/// - [`ConditionalTaskFrame`]
/// - [`SelectTaskFrame`]
/// - [`Task`]
#[async_trait]
pub trait TaskFrame: Send + Sync {
    /// The execution logic of the [`TaskFrame`] and subsequentially [`Task`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``ctx`` which is the context object,
    /// this context object is private and cannot be created by outside parties, but only in
    /// [`Task`]. The context wraps common information for the [`TaskFrame`] to access and pass
    /// on to other child [`TaskFrame`]s
    ///
    /// # Returns
    /// A ``Result<(), TaskError>`` which on success returns ``Ok(())`` (i.e. No result) and on
    /// failure it returns a ``Err(TaskError)`` indicating what went wrong on the [`TaskFrame`]
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskContext`]
    /// - [`TaskFrame`]
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError>;
}

#[async_trait]
impl<F> TaskFrame for F
where
    F: ?Sized + Deref + Send + Sync,
    F::Target: TaskFrame,
{
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        self.deref().execute(ctx).await
    }
}
