use crate::task::conditionframe::ConditionalFramePredicate;
use crate::task::dependency::FrameDependency;
use crate::task::retryframe::RetryBackoffStrategy;
use crate::task::{
    ConditionalFrame, ConstantBackoffStrategy, DependencyTaskFrame, FallbackTaskFrame,
    NoOperationTaskFrame, RetriableTaskFrame, TaskFrame, TimeoutTaskFrame,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// [`TaskFrameBuilder`] is a composable builder for constructing [`TaskFrame`] workflows, it wraps
/// a given [`TaskFrame`] and provides builder-style methods.
///
/// These methods add on top of the taskframe behavioral wrappers (such as retry, timeout, fallback,
/// condition, dependency, etc...), each method modifies the TaskFrame and returns the builder to
/// allow for continuous chaining.
///
/// The wrapping order matters: methods called **later** produce the **outermost** layer. For example:
///
/// For example ``TaskFrameBuilder::new(my_frame).with_retry(...).with_timeout(...)`` where "my_frame" is
/// your [`TaskFrame`] (lets call its type "MyFrame") produces as a type:
///
/// > ``TimeoutTaskFrame<RetriableTaskFrame<MyFrame>>``
///
/// Because "with_retry" wraps "MyFrame" first, then "with_timeout" wraps the result. In contrast, using
/// `TaskFrameBuilder::new(my_frame).with_timeout(...).with_retry(...)` produces:
///
/// > `RetriableTaskFrame<TimeoutTaskFrame<MyFrame>>`
///
/// Here, "with_timeout" wraps "MyFrame" first, and "with_retry" becomes the outer layer. Think of
/// it like function composition where `outer(inner(MyFrame))`. The last call is always the outermost
/// wrapper.
///
/// # Method(s)
/// - [`with_instant_retry`](TaskFrameBuilder::with_instant_retry) - Wraps with [`RetriableTaskFrame`] using zero-delay retries.
/// - [`with_retry`](TaskFrameBuilder::with_retry) - Wraps with [`RetriableTaskFrame`] with a constant delay between retries.
/// - [`with_backoff_retry`](TaskFrameBuilder::with_backoff_retry) - Wraps with [`RetriableTaskFrame`] using a custom [`RetryBackoffStrategy`].
/// - [`with_timeout`](TaskFrameBuilder::with_timeout) - Wraps with [`TimeoutTaskFrame`], cancelling execution if it exceeds the given duration.
/// - [`with_fallback`](TaskFrameBuilder::with_fallback) - Wraps with [`FallbackTaskFrame`], executing a secondary frame if the primary fails.
/// - [`with_condition`](TaskFrameBuilder::with_condition) - Wraps with [`ConditionalFrame`], only executing if the predicate is true (no-op otherwise).
/// - [`with_fallback_condition`](TaskFrameBuilder::with_fallback_condition) - Wraps with [`ConditionalFrame`], executing a fallback frame when the predicate is false.
/// - [`with_dependency`](TaskFrameBuilder::with_dependency) - Wraps with [`DependencyTaskFrame`], waiting for a single dependency before executing.
/// - [`with_dependencies`](TaskFrameBuilder::with_dependencies) - Wraps with [`DependencyTaskFrame`], waiting for multiple dependencies before executing.
/// - [`build`](TaskFrameBuilder::build) - Consumes the builder and returns the fully composed frame.
///
/// # Constructor(s)
/// The only constructor is [`TaskFrameBuilder::new`], which accepts any type implementing [`TaskFrame`]
/// and wraps it inside the builder to begin the chaining process.
///
/// # Accessing/Modifying Field(s)
/// The inner frame is not directly accessible. The only way to extract the composed frame is via
/// the [`build`](TaskFrameBuilder::build) method, which consumes the builder and returns the inner [`TaskFrame`].
///
/// # Trait Implementation(s)
/// [`TaskFrameBuilder`] does not implement any additional traits beyond the auto-derived ones. It is
/// intentionally a plain wrapper whose sole purpose is to provide the chaining API.
///
/// # Example(s)
/// ```
/// use std::num::NonZeroU32;
/// use std::time::Duration;
/// use chronographer::task::TaskFrameBuilder;
/// # use chronographer::task::{TaskFrame, TaskFrameContext, FallbackTaskFrame, TimeoutTaskFrame, RetriableTaskFrame};
/// # use async_trait::async_trait;
/// # use std::any::{Any, TypeId};
///
/// # struct MyFrame;
/// #
/// # #[async_trait]
/// # impl TaskFrame for MyFrame {
/// #     type Error = String;
/// #
/// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// # }
/// #
/// # struct BackupFrame;
/// #
/// # #[async_trait]
/// # impl TaskFrame for BackupFrame {
/// #     type Error = String;
/// #
/// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// # }
///
/// // `MyFrame` and `BackupFrame` are two types that implement `TaskFrame`.
///
/// const DELAY_PER_RETRY: Duration = Duration::from_secs(1);
/// # type WorkflowType = FallbackTaskFrame<TimeoutTaskFrame<RetriableTaskFrame<MyFrame>>, BackupFrame>;
/// # type WorkflowPermut1 = FallbackTaskFrame<RetriableTaskFrame<TimeoutTaskFrame<MyFrame>>, BackupFrame>;
/// # type WorkflowPermut2 = TimeoutTaskFrame<FallbackTaskFrame<RetriableTaskFrame<MyFrame>, BackupFrame>>;
///
/// let composed = TaskFrameBuilder::new(MyFrame)
///     .with_retry(NonZeroU32::new(3).unwrap(), DELAY_PER_RETRY) // Failure? Retry 3 times with 1s delay
///     .with_timeout(Duration::from_secs(30)) // Exceeded 30 seconds, terminate and error out with timeout?
///     .with_fallback(BackupFrame) // Received a timeout or another error? Run "BackupFrame"
///     .build();
///
/// # assert_eq!(composed.type_id(), TypeId::of::<WorkflowType>());
/// # assert_ne!(composed.type_id(), TypeId::of::<WorkflowPermut1>(), "Unexpected matching workflow types");
/// # assert_ne!(composed.type_id(), TypeId::of::<WorkflowPermut2>(), "Unexpected matching workflow types");
/// ```
/// With the workflow created, `composed` is now the type:
/// > ``FallbackTaskFrame<TimeoutTaskFrame<RetriableTaskFrame<MyFrame>>, BackupFrame>``
///
/// all from this builder, without the complexity of manually creating this type
///
/// # See Also
/// - [`TaskFrame`] - The core trait that defines execution logic.
/// - [`RetriableTaskFrame`] - The retry wrapper frame.
/// - [`TimeoutTaskFrame`] - The timeout wrapper frame.
/// - [`FallbackTaskFrame`] - The fallback wrapper frame.
/// - [`ConditionalFrame`] - The conditional execution wrapper frame.
/// - [`DependencyTaskFrame`] - The dependency-gated wrapper frame.
/// - [`Task`](crate::task::Task) - The top-level struct combining a frame with a trigger.
pub struct TaskFrameBuilder<T: TaskFrame>(T);

impl<T: TaskFrame> TaskFrameBuilder<T> {
    /// Method creates a new [`TaskFrameBuilder`] by wrapping the given [`TaskFrame`], this is the
    /// only entry point for constructing a builder for the workflow.
    ///
    /// The provided [`TaskFrame`] becomes the innermost layer of the composed workflow. Subsequent
    /// `with_*` calls wrap additional behavior around it, and [`build`](TaskFrameBuilder::build)
    /// extracts the final composed frame and builds complex workflows.
    ///
    /// # Argument(s)
    /// Any type implementing [`TaskFrame`], this becomes the base frame that all
    /// subsequent wrappers are layered on top of.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping `frame`, ready for chaining `with_*` methods.
    ///
    /// # Example(s)
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    /// # use chronographer::task::{TaskFrame, TaskFrameContext};
    /// # use async_trait::async_trait;
    /// #
    /// # struct MyFrame;
    /// #
    /// # #[async_trait]
    /// # impl TaskFrame for MyFrame {
    /// #     type Error = String;
    /// #
    /// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    ///
    /// // Wrap `MyFrame` in a builder, then immediately extract it unchanged.
    /// let frame: MyFrame = TaskFrameBuilder::new(MyFrame).build();
    /// ```
    /// When called without any `with_*` methods, [`build`](TaskFrameBuilder::build) returns
    /// the original frame as-is. In practice you would chain one or more wrappers before building for more complex workflows as per requirements.
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`TaskFrameBuilder::build`] - Consumes the builder and returns the composed frame.
    /// - [`TaskFrame`] - The trait that `frame` must implement.
    pub fn new(frame: T) -> Self {
        Self(frame)
    }
}

impl<T: TaskFrame> TaskFrameBuilder<T> {
    /// Method wraps the inner [`TaskFrame`] in a [`RetriableTaskFrame`] configured for instant retries.
    ///
    /// This wrapper allows the execution to immediately retry upon failure without any
    /// intermediate delay (backoff). It is particularly useful for fast-failing, transient
    /// issues where a delay would be unnecessary.
    ///
    /// # Arguments
    /// `retries` is a type [`NonZeroU32] parameter specifying the maximum number of times frame should retry on failure.
    /// even after retries, the workflow part may not be able to recover from the error and thus propegate it also task will be terminated.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with an immediate retry.
    ///
    /// # Example(s)
    /// ```
    /// use std::num::NonZeroU32;
    /// use chronographer::task::TaskFrameBuilder;
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, RetriableTaskFrame};
    /// # use async_trait::async_trait;
    /// #
    /// # struct MyFrame;
    /// #
    /// # #[async_trait]
    /// # impl TaskFrame for MyFrame {
    /// #     type Error = String;
    /// #
    /// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    ///
    /// let retries = NonZeroU32::new(3).unwrap();
    /// let builder = TaskFrameBuilder::new(MyFrame)
    ///     .with_instant_retry(retries) // Retries up to 3 times on failure
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`RetriableTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame
    /// - [`TaskFrame`] - The trait that `frame` must implement.
    pub fn with_instant_retry(
        self,
        retries: NonZeroU32,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(
            RetriableTaskFrame::builder()
                .retries(retries)
                .frame(self.0)
                .build(),
        )
    }

    /// Method wraps the inner [`TaskFrame`] in a [`RetriableTaskFrame`] configured with a constant delay between retries.
    ///
    /// This wrapper allows the execution to retry upon failure with a constant delay between attempts. It is useful for
    /// retrying with a fixed interval between retries.
    ///
    /// # Arguments
    ///
    /// - `retries` is a type [`NonZeroU32`] parameter specifying the maximum number of times frame should retry on failure.
    ///   even after retries, the workflow part may not be able to recover from the error and thus propegate it also task will be terminated.
    /// - `delay` is a type [`Duration`] parameter specifying the constant delay between retries.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a retry configured with a constant delay per retry.
    ///
    /// # Examples
    /// ```
    /// use chrono_grapher::task::{TaskFrameBuilder, NonZeroU32, Duration};
    /// use std::time::Duration;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, RetriableTaskFrame};
    /// # use async_trait::async_trait;
    /// #
    /// # struct MyFrame;
    /// #
    /// # #[async_trait]
    /// # impl TaskFrame for MyFrame {
    /// #     type Error = String;
    /// #
    /// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    ///
    /// let retries = NonZeroU32::new(3).unwrap();
    /// let delay_per_retry = Duration::from_secs(1);
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_retry(retries, delay_per_retry)
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`RetriableTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame
    /// - [`TaskFrame`] - The trait that `frame` must implement.
    pub fn with_retry(
        self,
        retries: NonZeroU32,
        delay: Duration,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(
            RetriableTaskFrame::builder()
                .retries(retries)
                .frame(self.0)
                .backoff(ConstantBackoffStrategy::new(delay))
                .build(),
        )
    }
    
    /// Method wraps the inner [`TaskFrame`] in a [`RetriableTaskFrame`] configured with a custom backoff strategy between retries.
    ///
    /// This wrapper allows the execution to retry upon failure using a custom [`RetryBackoffStrategy`] to determine
    /// the delay between attempts. It is useful for scenarios requiring exponential backoff, jitter, or other dynamic
    /// retry intervals.
    ///
    /// # Arguments
    ///
    /// - `retries` is a type [`NonZeroU32`] parameter specifying the maximum number of times frame should retry on failure.
    ///   even after retries, the workflow part may not be able to recover from the error and thus propegate it also task will be terminated.
    /// - `strat` is a type implementing [`RetryBackoffStrategy`] parameter specifying the custom backoff strategy between retries.
    /// 
    /// # Strategy
    ///
    /// - [`ConstantBackoffStrategy`] this backoff strategy retries execution with a constant delay duration between attempts.
    /// - [`LinearBackoffStrategy`] this backoff strategy retries execution with a delay that scales linearly (delay * retry attempt).
    /// - [`ExponentialBackoffStrategy`] this backoff strategy retries execution with a delay that scales exponentially (delay ^ retry attempt).
    ///  
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a retry configured with the provided backoff strategy.
    ///
    /// # Examples
    /// ```
    /// use std::num::NonZeroU32;
    /// use std::time::Duration;
    /// use chronographer::task::{TaskFrameBuilder, ConstantBackoffStrategy};
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, RetriableTaskFrame};
    /// # use async_trait::async_trait;
    /// #
    /// # struct MyFrame;
    /// #
    /// # #[async_trait]
    /// # impl TaskFrame for MyFrame {
    /// #     type Error = String;
    /// #
    /// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    ///
    /// let retries = NonZeroU32::new(3).unwrap();
    /// let strategy = ConstantBackoffStrategy::new(Duration::from_secs(1));
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_backoff_retry(retries, strategy)
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`RetriableTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame
    /// - [`RetryBackoffStrategy`] - The trait that `strat` must implement.
    /// - [`TaskFrame`] - The trait that `frame` must implement.
    pub fn with_backoff_retry(
        self,
        retries: NonZeroU32,
        strat: impl RetryBackoffStrategy,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(
            RetriableTaskFrame::builder()
                .retries(retries)
                .frame(self.0)
                .backoff(strat)
                .build(),
        )
    }

    pub fn with_timeout(self, max_duration: Duration) -> TaskFrameBuilder<TimeoutTaskFrame<T>> {
        TaskFrameBuilder(TimeoutTaskFrame::new(self.0, max_duration))
    }

    pub fn with_fallback<T2: TaskFrame + 'static>(
        self,
        fallback: T2,
    ) -> TaskFrameBuilder<FallbackTaskFrame<T, T2>> {
        TaskFrameBuilder(FallbackTaskFrame::new(self.0, fallback))
    }

    pub fn with_condition(
        self,
        predicate: impl ConditionalFramePredicate + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T, NoOperationTaskFrame<T::Error>>> {
        let condition = ConditionalFrame::builder()
            .predicate(predicate)
            .frame(self.0)
            .error_on_false(false)
            .build();
        TaskFrameBuilder(condition)
    }

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

    async fn with_dependency(
        self,
        dependency: impl FrameDependency + 'static,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .frame(self.0)
            .dependencies(vec![Arc::new(dependency)])
            .build();

        TaskFrameBuilder(dependent)
    }

    async fn with_dependencies(
        self,
        dependencies: Vec<Arc<dyn FrameDependency>>,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .frame(self.0)
            .dependencies(dependencies)
            .build();

        TaskFrameBuilder(dependent)
    }

    pub fn build(self) -> T {
        self.0
    }
}
