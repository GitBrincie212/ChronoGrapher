//! Contains the utility [`TaskFrameBuilder`] which is used for constructing workflows via
//! a builder-based pattern. For more information on how its used check the documentation of [`TaskFrameBuilder`]

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
    /// the original frame as-is. In practice, you would chain one or more wrappers before building for more complex workflows as per requirements.
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
    /// - ``retries`` is a type [`NonZeroU32`] parameter specifying the maximum number of times, the TaskFrame should retry on failure.
    ///   even after retries, the workflow part may not be able to recover from the error and thus propegate it also task will be terminated.
    /// - ``strat`` is a type implementing [`RetryBackoffStrategy`] parameter specifying the custom backoff strategy between retries.
    ///
    /// ChronoGrapher currently provides these three backoff strategies but new ones may be derived via [`RetryBackoffStrategy`]:
    /// - [`ConstantBackoffStrategy`] - Retries execution with a constant delay duration between attempts.
    /// - [`LinearBackoffStrategy`] - Retries execution with a delay that scales linearly (``delay`` * ``retry_attempt``).
    /// - [`ExponentialBackoffStrategy`] - Retries execution with a delay that scales exponentially (``delay`` ^ ``retry_attempt``).
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
    /// - [`RetryBackoffStrategy`] - The trait that ``strat`` must implement.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
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

    /// Method wraps the inner [`TaskFrame`] in a [`TimeoutTaskFrame`] which will timeout and cancel execution if the inner task exceeds the specified duration.
    ///
    /// This wrapper allows the execution to be strictly bound by a time limit. If the inner task takes longer than the
    /// ``max_duration`` parameter, it will be forcefully yielded, canceled, and a timeout error will be propagated up the chain.
    ///
    /// > **Note:** Due to limitations from Rust, the [`TimeoutTaskFrame``] might not cancel in time the operation especially if its CPU-heavy work, for this reason it is
    /// reccomended to ``yield`` whenever possible. For more information visit the [`TimeoutTaskFrame`] documentation.
    ///
    /// # Arguments
    /// ``max_duration`` is a type [`Duration`] parameter specifying the maximum amount of time the task is allowed to run before being timed out.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a timeout limit.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use chronographer::task::TaskFrameBuilder;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, TimeoutTaskFrame};
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
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_timeout(Duration::from_secs(30)) // Give the inner task up to 30 seconds to finish
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`TimeoutTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
    pub fn with_timeout(self, max_duration: Duration) -> TaskFrameBuilder<TimeoutTaskFrame<T>> {
        TaskFrameBuilder(TimeoutTaskFrame::new(self.0, max_duration))
    }

    /// Method wraps the inner [`TaskFrame`] in a [`FallbackTaskFrame`] which will execute a specified fallback task upon failure of the main task.
    ///
    /// This wrapper allows for fault tolerance by providing an alternative workflow path. If the primary inner task returns an error,
    /// the assigned fallback frame will automatically be executed instead, preventing the workflow from immediately failing.
    ///
    /// # Arguments
    ///
    /// ``fallback`` is a type implementing [`TaskFrame`] parameter specifying the alternative task to execute if the primary task fails.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a fallback behavior.
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, FallbackTaskFrame};
    /// # use async_trait::async_trait;
    /// #
    /// # struct MyFrame;
    /// #
    /// # #[async_trait]
    /// # impl TaskFrame for MyFrame {
    /// #     type Error = String;
    /// #
    /// #     async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
    /// #         Err("Primary task failed".to_string())
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
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_fallback(BackupFrame) // Run BackupFrame if MyFrame fails
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`FallbackTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`TaskFrame`] - The trait that ``fallback`` must implement.
    pub fn with_fallback<T2: TaskFrame + 'static>(
        self,
        fallback: T2,
    ) -> TaskFrameBuilder<FallbackTaskFrame<T, T2>> {
        TaskFrameBuilder(FallbackTaskFrame::new(self.0, fallback))
    }

    /// Method wraps the inner [`TaskFrame`] in a [`ConditionalFrame`] which conditionally executes the task based on a provided predicate function.
    ///
    /// This wrapper allows the execution of the inner task to be controlled by dynamic condition logic. The predicate is
    /// evaluated asynchronously to determine whether the inner task should run. Note that the predicate is not a direct
    /// boolean value but rather a logic wrapper (implementing [`ConditionalFramePredicate`]) that gets checked at runtime.
    ///
    /// If the predicate returns `true`, the inner task is unconditionally executed, however if it returns `false`, in the context 
    /// of this method, it acts as a no-operation and returns a success by default upon a falsey value.
    ///
    /// If a fallback behaiviour needs to be configured to execute as a backup when the predicate returns false, then 
    /// [`with_fallback_condition`](TaskFrameBuilder::with_fallback_condition) is the better choice.
    ///
    /// # Arguments
    /// The method requires one argument that being ``predicate`` is a type implementing [`ConditionalFramePredicate`] parameter containing the condition logic to evaluate.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a conditional execution gate.
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, ConditionalFramePredicate, RestrictTaskFrameContext};
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
    /// #
    /// # struct CheckCondition;
    /// #
    /// # #[async_trait]
    /// # impl ConditionalFramePredicate for CheckCondition {
    /// #     async fn execute(&self, _ctx: &RestrictTaskFrameContext) -> bool {
    /// #         true
    /// #     }
    /// # }
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_condition(CheckCondition) // Execute MyFrame only if the predicate executes to true
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`ConditionalFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`ConditionalFramePredicate`] - The core trait that defines the condition evaluation.
    /// - [`with_fallback_condition`](TaskFrameBuilder::with_fallback_condition) - Wrapper that executes a fallback frame on false conditions.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
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

    /// Method wraps the inner [`TaskFrame`] in a [`ConditionalFrame`] and optionally executes a secondary inner task upon a falsey condition value.
    ///
    /// This wrapper allows the execution of the inner task to be controlled by dynamic condition logic. The predicate is
    /// evaluated asynchronously to determine whether the inner task should run. Note that the predicate is not a direct
    /// boolean value but rather a logic wrapper (implementing [`ConditionalFramePredicate`]) that gets checked at runtime.
    ///
    /// If the predicate returns ``true``, the inner task is unconditionally executed, however if it returns ``false``, in the context 
    /// of this method, it executes a ``fallback`` [`TaskFrame`] defined in the arguments and returns a success by default upon a falsey value.
    ///
    /// If an error is desired to be returned and without a fallback [`TaskFrame`] executing on top, then 
    /// [`with_condition`](TaskFrameBuilder::with_condition) is the better choice.
    ///
    /// # Arguments
    /// There are two arguments the method requires, the first is ``fallback`` which is a type implementing [`TaskFrame`] parameter specifying the alternative/secondary 
    /// task to execute if the conditional predicate evaluates to false.
    ///
    /// The second is the ``predicate`` is a type implementing [`ConditionalFramePredicate`] parameter containing the logic to evaluate.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a conditional-fallback execution gate.
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, ConditionalFramePredicate, RestrictTaskFrameContext};
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
    /// #
    /// # struct CheckCondition;
    /// #
    /// # #[async_trait]
    /// # impl ConditionalFramePredicate for CheckCondition {
    /// #     async fn execute(&self, _ctx: &RestrictTaskFrameContext) -> bool {
    /// #         false
    /// #     }
    /// # }
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_fallback_condition(BackupFrame, CheckCondition) // Runs MyFrame if true, BackupFrame if false
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`ConditionalFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`ConditionalFramePredicate`] - The core trait that defines the condition evaluation.
    /// - [`with_condition`](TaskFrameBuilder::with_condition) - Wrapper that simply aborts/skips on false condition.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
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

    /// Method wraps the inner [`TaskFrame`] in a [`DependencyTaskFrame`] waiting for a specific dependency to be resolved before execution.
    ///
    /// This wrapper allows the execution of the inner task to be deferred until a defined condition or dependency
    /// is met. It polls the provided [`FrameDependency`] asynchronously until it indicates that it has been resolved,
    /// only then allowing the primary inner task to execute.
    ///
    /// If one desires to specify multiple dependencies instead of only one, then [`with_dependencies`](TaskFrameBuilder::with_dependencies) is the method for this requirement.
    ///
    /// # Arguments
    /// The method requires one argument, that being ``dependency`` is a type implementing the [`FrameDependency`] trait that guards the inner task's execution.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a dependency execution gate.
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    /// use std::sync::atomic::AtomicBool;
    /// use std::sync::Arc;
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, FlagDependency};
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
    /// // Create a simple flag dependency that can be toggled
    /// let atomic_flag = Arc::new(AtomicBool::new(false));
    /// let flag_dep = FlagDependency::new(atomic_flag.clone());
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_dependency(flag_dep) // MyFrame will only execute when the flag resolves to true
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`DependencyTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`FrameDependency`] - The core trait that defines the required dependency.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
    pub fn with_dependency(
        self,
        dependency: impl FrameDependency + 'static,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .frame(self.0)
            .dependencies(vec![Arc::new(dependency)])
            .build();

        TaskFrameBuilder(dependent)
    }

    /// Method wraps the inner [`TaskFrame`] in a [`DependencyTaskFrame`] waiting for multiple dependencies to be resolved before execution.
    ///
    /// This behaves exactly like [`with_dependency`](TaskFrameBuilder::with_dependency), but it takes a collection of [`FrameDependency`] instances.
    /// The inner task will only begin execution once **all** the provided dependencies have been successfully resolved.
    ///
    /// Method wraps the inner [`TaskFrame`] in a [`DependencyTaskFrame`] waiting for multiple dependencies to be resolved before execution.
    ///
    /// This wrapper allows the execution of the inner task to be deferred until multiple defined condition or dependencies are met. It polls 
    /// the provided [`FrameDependencies`](FrameDependency) asynchronously until all indicate that they've been resolved,
    /// only then allowing the primary inner task to execute.
    ///
    /// If one desires to specify only one dependency instead of multiple, then [`with_dependency`](TaskFrameBuilder::with_dependency) is the method for this requirement.
    ///
    /// # Arguments
    /// The method accepts one argument, that being ``dependencies`` is a ``Vec`` of elements implementing the [`FrameDependency`] trait, acting as multiple guards for the inner task's execution.
    ///
    /// # Returns
    /// A [`TaskFrameBuilder`] wrapping its inner workflow with a multi-dependency execution gate.
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    /// use std::sync::atomic::AtomicBool;
    /// use std::sync::Arc;
    /// # use std::sync::atomic::Ordering;
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, FlagDependency, FrameDependency};
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
    /// let atomic_flag1 = Arc::new(AtomicBool::new(false));
    /// let atomic_flag2 = Arc::new(AtomicBool::new(false));
    ///
    /// let dep1 = Arc::new(FlagDependency::new(atomic_flag1.clone())) as Arc<dyn FrameDependency>;
    /// let dep2 = Arc::new(FlagDependency::new(atomic_flag2.clone())) as Arc<dyn FrameDependency>;
    ///
    /// let task = TaskFrameBuilder::new(MyFrame)
    ///     .with_dependencies(vec![dep1, dep2]) // Executes when both dependencies resolve
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`DependencyTaskFrame`] - The TaskFrame component which wraps the innermost TaskFrame.
    /// - [`FrameDependency`] - The core trait that defines the required dependency.
    /// - [`TaskFrame`] - The trait that ``frame`` must implement.
    pub fn with_dependencies(
        self,
        dependencies: Vec<Arc<dyn FrameDependency>>,
    ) -> TaskFrameBuilder<DependencyTaskFrame<T>> {
        let dependent: DependencyTaskFrame<T> = DependencyTaskFrame::builder()
            .frame(self.0)
            .dependencies(dependencies)
            .build();

        TaskFrameBuilder(dependent)
    }

    /// Method consumes the builder and returns the underlying, fully-composed [`TaskFrame`].
    ///
    /// This method serves as the final step in the builder chain. After stacking various behaviors
    /// (such as retries, timeouts, or conditions) on top of the inner task, `build` extracts the
    /// final constructed workflow so it can be executed or embedded inside a `Task`.
    ///
    /// # Returns
    /// The composed inner [`TaskFrame`] of type ``T`` (containing the inner TaskFrame with the additional behaviors defined).
    ///
    /// # Examples
    /// ```
    /// use chronographer::task::TaskFrameBuilder;
    /// use std::time::Duration;
    ///
    /// # use chronographer::task::{TaskFrame, TaskFrameContext, TimeoutTaskFrame};
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
    /// let base_frame = MyFrame;
    ///
    /// // Using the builder to wrap it in a timeout and then extracting it
    /// let built_frame: TimeoutTaskFrame<MyFrame> = TaskFrameBuilder::new(base_frame)
    ///     .with_timeout(Duration::from_secs(5))
    ///     .build(); // <- Returns the fully composed frame, discarding the builder
    /// ```
    ///
    /// # See Also
    /// - [`TaskFrameBuilder`] - The main builder which the method is part of.
    /// - [`TaskFrame`] - The trait that the returned frame implements.
    pub fn build(self) -> T {
        self.0
    }
}
