use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use crate::task::{ConditionalFrame, DependencyTaskFrame, FallbackTaskFrame, RetriableTaskFrame, TaskFrame, TimeoutTaskFrame};
use crate::task::conditionframe::ConditionalFramePredicate;
use crate::task::dependency::FrameDependency;
use crate::task::retryframe::RetryBackoffStrategy;

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
    /// - [`ConditionalFramePredicate`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_fallback_condition`]
    pub fn with_condition(
        self,
        predicate: impl ConditionalFramePredicate + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T>> {
        let condition: ConditionalFrame<T> = ConditionalFrame::<T>::builder()
            .predicate(predicate)
            .frame(self.0)
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
    /// - [`ConditionalFramePredicate`]
    /// - [`TaskFrameBuilder`]
    /// - [`TaskFrameBuilder::with_condition`]
    pub fn with_fallback_condition<T2: TaskFrame + 'static>(
        self,
        fallback: T2,
        predicate: impl ConditionalFramePredicate + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T, T2>> {
        let condition: ConditionalFrame<T, T2> = ConditionalFrame::<T, T2>::fallback_builder()
            .predicate(predicate)
            .frame(self.0)
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