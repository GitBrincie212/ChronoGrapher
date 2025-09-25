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
use crate::task::conditionframe::FramePredicateFunc;
use crate::task::dependency::FrameDependency;
use crate::task::events::TaskEventEmitter;
use crate::task::retryframe::RetryBackoffStrategy;
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
use std::num::{NonZeroU32, NonZeroU64};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
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
/// by the developer) til it succeeds, or it reaches this threshold, with a specified backoff retry strategy
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

/// [`TaskFrameBuilder`] acts more as a utility rather than a full feature, it allows to construct
/// the default implemented task frames with a more builder syntax
///
/// # Constructor(s)
/// One can instantiate [`TaskFrameBuilder`] via [`TaskFrameBuilder::new`]
///
/// # Example
/// ```ignore
/// use std::num::NonZeroU32;
/// use std::time::Duration;
/// use chronographer_core::task::{ExecutionTaskFrame, TaskFrameBuilder};
///
/// let simple_frame = ExecutionTaskFrame::new(|_| async {Ok(())});
///
/// let frame = TaskFrameBuilder::new(simple_frame)
///     .with_timeout(Duration::from_secs_f64(2.32))
///     .with_retry(NonZeroU32::new(15).unwrap(), Duration::from_secs_f64(1.0))
///     .with_condition(|metadata| {
///         metadata.runs() % 2 == 0
///     })
///     .build();
///
/// // While the builder approach alleviates the more cumbersome
/// // writing of the common approach, it doesn't allow custom
/// // task frames implemented from third parties (you can
/// // mitigate this somewhat with the new-type pattern)
/// ```
pub struct TaskFrameBuilder<T: TaskFrame>(T);

impl<T: TaskFrame> TaskFrameBuilder<T> {
    /// Constructs / Creates a new [`TaskFrameBuilder`] from a provided
    /// [`TaskFrame`]
    ///
    /// # Argument(s)
    /// This method accepts only one argument, that being a [`TaskFrame`] to begin
    /// the building process of stacking task frames on top of it
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`TaskFrameBuilder`]
    pub fn new(frame: T) -> Self {
        Self(frame)
    }

    /// Stacks this [`TaskFrame`] with an instant [`RetriableTaskFrame`], i.e.
    /// it retries the task frame a total of ``retries`` specified times, and each retry is an instant
    ///
    /// There are also versions of this method such as:
    /// - [`TaskFrameBuilder::with_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``retries``, which is the number of times to
    /// attempt the execution of a task frame til it succeeds or til the counter runs out
    ///
    /// # Return(s)
    /// A modified builder that wraps [`RetriableTaskFrame`] around the current [`TaskFrame`] with
    /// the number of retries being ``retries`` and no delay per retry
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetriableTaskFrame`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    pub fn with_instant_retry(
        self,
        retries: NonZeroU32,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(RetriableTaskFrame::new_instant(self.0, retries))
    }

    /// Stacks this [`TaskFrame`] with a delayed [`RetriableTaskFrame`], i.e.
    /// it retries the task frame a total of ``retries`` specified times, and each retry has a
    /// specific delay
    ///
    /// There are also versions of this method such as:
    /// - [`TaskFrameBuilder::with_instant_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being ``retries`` and ``delay``, the former is
    /// the number of times to attempt the execution of a task frame til it succeeds or til
    /// the counter runs out. While the latter is the amount of delay between retries
    ///
    /// # Return(s)
    /// A modified builder that wraps [`RetriableTaskFrame`] around the current [`TaskFrame`] with
    /// the number of retries being ``retries`` and the delay per retry being ``delay``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetriableTaskFrame`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_instant_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    pub fn with_retry(
        self,
        retries: NonZeroU32,
        delay: Duration,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(RetriableTaskFrame::new(self.0, retries, delay))
    }

    /// Stacks this [`TaskFrame`] with a [`RetriableTaskFrame`] that contains a retry backoff strategy,
    /// i.e. it retries the task frame a total of ``retries`` specified times, and each retry, a new
    /// delay is computed to be used
    ///
    /// There are also versions of this method such as:
    /// - [`TaskFrameBuilder::with_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being ``retries`` and ``strat``, the former is
    /// the number of times to attempt the execution of a task frame til it succeeds or til
    /// the counter runs out. While the latter is an implementation of the [`RetryBackoffStrategy`]
    /// which computes the delay between retries
    ///
    /// # Return(s)
    /// A modified builder that wraps [`RetriableTaskFrame`] around the current [`TaskFrame`] with
    /// the number of retries being ``retries`` and a retry backoff strategy being ``strat``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetriableTaskFrame`]
    /// - [`RetryBackoffStrategy`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_retry`]
    /// - [`TaskFrameBuilder::with_backoff_retry`]
    pub fn with_backoff_retry<T2: RetryBackoffStrategy>(
        self,
        retries: NonZeroU32,
        strat: T2,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T, T2>>
    where
        RetriableTaskFrame<T, T2>: TaskFrame,
    {
        TaskFrameBuilder(RetriableTaskFrame::<T, T2>::new_with(
            self.0, retries, strat,
        ))
    }

    /// Stacks this [`TaskFrame`] with a [`TimeoutTaskFrame`] that timeouts after it reaches a specific
    /// threshold, i.e. it runs the task frame, and if the task takes longer than a specified duration, then
    /// it returns a timeout error and halts the task (there is a limitation which should be checked in
    /// [`TimeoutTaskFrame`])
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``max_duration`` which specifies the
    /// maximum duration allowed for the [`TaskFrame`] to take, if it finishes sooner than the
    /// maximum duration, it does not do anything. However, if it does not, then it automatically
    /// stops the execution
    ///
    /// # Return(s)
    /// A modified builder that wraps [`TimeoutTaskFrame`] around the current [`TaskFrame`] with
    /// the maximum duration being ``max_duration``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`TimeoutTaskFrame`]
    /// - [`TaskFrameBuilder`]
    pub fn with_timeout(self, max_duration: Duration) -> TaskFrameBuilder<TimeoutTaskFrame<T>> {
        TaskFrameBuilder(TimeoutTaskFrame::new(self.0, max_duration))
    }

    /// Stacks this [`TaskFrame`] with a [`FallbackTaskFrame`] that fallbacks to a second specified
    /// task frame, i.e. it runs the task frame and if it succeeds, then it returns the results
    /// from the wrapped task frame, otherwise it executes the specified fallback task frame to
    /// be executed, returning the results from there
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``fallback`` which specifies the fallback
    /// [`TaskFrame`] in case the current [`TaskFrame`] fails to be executed after
    ///
    /// # Return(s)
    /// A modified builder that wraps [`FallbackTaskFrame`] around the current [`TaskFrame`] with
    /// the fallback task frame being ``fallback``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`FallbackTaskFrame`]
    /// - [`TaskFrameBuilder`]
    pub fn with_fallback<T2: TaskFrame + 'static>(
        self,
        fallback: T2,
    ) -> TaskFrameBuilder<FallbackTaskFrame<T, T2>> {
        TaskFrameBuilder(FallbackTaskFrame::new(self.0, fallback))
    }

    /// Stacks this [`TaskFrame`] with a [`ConditionalFrame`] that conditionally executes it based
    /// on a predicate, i.e. when the task frame tries to run, it runs a predicate and determines
    /// if it should run the task frame based on if the boolean value returned from the predicate is
    /// true, if false then it does not run it and returns a success instead
    ///
    /// There is an alternative version for specifying any fallback to execute if the predicate
    /// returns false, via [`TaskFrameBuilder::with_fallback_condition`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``predicate`` which specifies the predicate
    /// to be executed to determine the execution of [`TaskFrame`]
    ///
    /// # Return(s)
    /// A modified builder that wraps [`ConditionalFrame`] around the current [`TaskFrame`] with
    /// the predicate being ``predicate``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`ConditionalFrame`]
    /// - [`FramePredicateFunc`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_fallback_condition`]
    pub fn with_condition(
        self,
        predicate: impl FramePredicateFunc + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T>> {
        let condition: ConditionalFrame<T> = ConditionalFrame::<T>::builder()
            .predicate(predicate)
            .task(self.0)
            .error_on_false(false)
            .build();
        TaskFrameBuilder(condition)
    }

    /// Stacks this [`TaskFrame`] with a [`ConditionalFrame`] that conditionally executes it based
    /// on a predicate, if it returns false, it executes a fallback, i.e. when the task frame tries to run,
    /// it runs a predicate and determines if it should run the task frame based on if the
    /// boolean value returned from the predicate is true, if false then it does not run the wrapped
    /// task frame it but instead running a fallback in its place
    ///
    /// There is an alternative version for specifying to not execute anything if the predicate
    /// returns false, via [`TaskFrameBuilder::with_condition`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being ``fallback`` and ``predicate`` the former
    /// specifies the predicate to be executed to determine the execution of [`TaskFrame`] while
    /// the latter will be a [`TaskFrame`] executed when the ``predicate`` returns false
    ///
    /// # Return(s)
    /// A modified builder that wraps [`ConditionalFrame`] around the current [`TaskFrame`] with
    /// the predicate being ``predicate`` and a fallback in case ``predicate`` returns false being
    /// ``fallback``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`ConditionalFrame`]
    /// - [`FramePredicateFunc`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_condition`]
    pub fn with_fallback_condition<T2: TaskFrame + 'static>(
        self,
        fallback: T2,
        predicate: impl FramePredicateFunc + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T, T2>> {
        let condition: ConditionalFrame<T, T2> = ConditionalFrame::<T, T2>::fallback_builder()
            .predicate(predicate)
            .task(self.0)
            .fallback(fallback)
            .error_on_false(false)
            .build();
        TaskFrameBuilder(condition)
    }

    /// Stacks this [`TaskFrame`] with a [`DependencyTaskFrame`] that executes it based
    /// on if the one dependency, i.e. when the task frame tries to run,
    /// it checks if its only dependency is resolved if it is resolved then it executes it,
    /// otherwise it returns an error indicating that the dependency have not been resolved
    ///
    /// There is an alternative version for specifying more than one dependency to be resolved
    /// at the same time, that being [`TaskFrameBuilder::with_dependencies`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``dependency``
    /// which is an implementation of the [`FrameDependency`]
    ///
    /// # Return(s)
    /// A modified builder that wraps [`DependencyTaskFrame`] around the current [`TaskFrame`] with
    /// the dependencies being only one and that is ``dependency``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`DependencyTaskFrame`]
    /// - [`FrameDependency`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_dependencies`]
    #[allow(unused)]
    async fn with_dependency(
        self,
        dependency: impl FrameDependency + 'static,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .task(self.0)
            .dependencies(vec![Arc::new(dependency)])
            .build();

        TaskFrameBuilder(dependent)
    }

    /// Stacks this [`TaskFrame`] with a [`DependencyTaskFrame`] that executes it based
    /// on if multiple dependencies are resolved at the same time, i.e. when the task frame tries to
    /// run, it checks if all dependencies tied are resolved, if they are resolved then it executes it,
    /// otherwise it returns an error indicating that dependencies have not been resolved
    ///
    /// There is an alternative version for specifying only one dependency to be resolved
    /// that being [`TaskFrameBuilder::with_dependency`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``dependencies`` which is a ``Vec`` hosting
    /// all the dependencies which implement [`FrameDependency`], that have to be resolved at the
    /// same time in order to execute
    ///
    /// # Return(s)
    /// A modified builder that wraps [`DependencyTaskFrame`] around the current [`TaskFrame`] with
    /// all dependencies to be resolved being ``dependencies``
    ///
    /// # Method Behavior
    /// This builder method, unlike most builders, can be chained together with other
    /// methods to create complex task frames way more easily
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`DependencyTaskFrame`]
    /// - [`FrameDependency`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_dependency`]
    #[allow(unused)]
    async fn with_dependencies(
        self,
        dependencies: Vec<Arc<dyn FrameDependency>>,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .task(self.0)
            .dependencies(dependencies)
            .build();

        TaskFrameBuilder(dependent)
    }

    /// Builds the [`TaskFrame`] and returns it
    ///
    /// # Returns
    /// The fully complete [`TaskFrame`] instance based on
    /// the chained builder methods
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`TaskFrameBuilder`]
    pub fn build(self) -> T {
        self.0
    }
}
