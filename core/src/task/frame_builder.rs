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

/// [`TaskFrameBuilder`] is a composable builder for constructing [`TaskFrame`] workflows. It wraps a given [`TaskFrame`] and provides builder-style methods which add on top of this taskframe behavioral wrappers (such as retry, timeout, fallback, condition, dependency, etc.). Each method modifies the TaskFrame and returns the builder to allow for chaining.
///
/// The wrapping order matters: methods called **later** produce the **outermost** layer. For example:
///
/// For example `TaskFrameBuilder::new(my_frame).with_retry(...).with_timeout(...)` where "my_frame" is
/// your [`TaskFrame`] (lets call its type "MyFrame") produces as a type:
///
/// > `TimeoutTaskFrame<RetriableTaskFrame<MyFrame>>`
///
/// Because "with_retry" wraps "MyFrame" first, then "with_timeout" wraps the result. In contrast, using
/// `TaskFrameBuilder::new(my_frame).with_timeout(...).with_retry(...)` produces:
///
/// > `RetriableTaskFrame<TimeoutTaskFrame<MyFrame>>`
///
/// Here, "with_timeout" wraps "MyFrame" first, and "with_retry" becomes the outer layer. Think of it like function composition where `outer(inner(MyFrame))`. The last call is always the outermost wrapper.
///
/// `T: TaskFrame` - The inner frame type held by the builder at start. with every chaining call encoding the full nesting structure at the type level. for example
///  after calling ".with_retry(...)", the builder becomes `TaskFrameBuilder<RetriableTaskFrame<T>>`.
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
/// the [`build`](TaskFrameBuilder::build) method, which consumes the builder and returns the inner `T`.
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
///
/// // `MyFrame` and `BackupFrame` are two types that implement `TaskFrame`.
///
/// const DELAY_PER_RETRY: Duration = Duration::from_secs(1);
///
/// let composed = TaskFrameBuilder::new(MyFrame)
///     .with_retry(NonZeroU32::new(3).unwrap(), DELAY_PER_RETRY) // Failure? Retry 3 times with 1s delay
///     .with_timeout(Duration::from_secs(30)) // Exceeded 30 seconds, terminate and error out with timeout?
///     .with_fallback(BackupFrame) // Received a timeout or another error? Run "BackupFrame"
///     .build();
/// ```
/// With the workflow created, `composed` is now the type:
/// > ``FallbackTaskFrame<TimeoutTaskFrame<RetriableTaskFrame<MyFrame>>, BackupFrame>``
///
/// all from this builder, without the complexity of manually creating this type
///
///
/// # See Also
/// - [`TaskFrame`] — The core trait that defines execution logic.
/// - [`RetriableTaskFrame`] — The retry wrapper frame.
/// - [`TimeoutTaskFrame`] — The timeout wrapper frame.
/// - [`FallbackTaskFrame`] — The fallback wrapper frame.
/// - [`ConditionalFrame`] — The conditional execution wrapper frame.
/// - [`DependencyTaskFrame`] — The dependency-gated wrapper frame.
/// - [`Task`](crate::task::Task) — The top-level struct combining a frame with a trigger.

pub struct TaskFrameBuilder<T: TaskFrame>(T);

impl<T: TaskFrame> TaskFrameBuilder<T> {
    pub fn new(frame: T) -> Self {
        Self(frame)
    }
}

impl<T: TaskFrame> TaskFrameBuilder<T> {
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
