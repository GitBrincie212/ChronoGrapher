use crate::task::conditionframe::ConditionalFramePredicate;
use crate::task::dependency::FrameDependency;
use crate::task::retryframe::RetryBackoffStrategy;
use crate::task::{ConditionalFrame, ConstantBackoffStrategy, DependencyTaskFrame, FallbackTaskFrame, NoOperationTaskFrame, RetriableTaskFrame, TaskFrame, TimeoutTaskFrame};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

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

    #[allow(unused)]
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

    #[allow(unused)]
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
