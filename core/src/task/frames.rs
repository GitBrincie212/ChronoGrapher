pub mod conditionframe;
pub mod dependencyframe;
pub mod executionframe;
pub mod fallbackframe;
pub mod parallelframe;
pub mod retryframe;
pub mod selectframe;
pub mod sequentialframe;
pub mod timeoutframe;

use crate::task::conditionframe::FramePredicateFunc;
use crate::task::dependency::FrameDependency;
use crate::task::events::TaskEventEmitter;
use crate::task::retryframe::RetryBackoffStrategy;
use crate::task::{TaskEndEvent, TaskError, TaskMetadata, TaskStartEvent};
use async_trait::async_trait;
pub use conditionframe::ConditionalFrame;
pub use dependencyframe::DependencyTaskFrame;
pub use executionframe::ExecutionTaskFrame;
pub use fallbackframe::FallbackTaskFrame;
pub use parallelframe::ParallelTaskFrame;
pub use retryframe::RetriableTaskFrame;
pub use selectframe::SelectTaskFrame;
pub use sequentialframe::SequentialTaskFrame;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
pub use timeoutframe::TimeoutTaskFrame;

/// [`TaskFrame`] represents a unit of work which hosts the actual computation logic for the [`Scheduler`]
/// to invoke, this is a part of the task system. [`TaskFrame`] encapsulates mainly the async execution logic
/// of the task which is evaluated by calling [`TaskFrame::execute`] Returning `Ok(())` on success,
/// or an `Arc<dyn Error>` on failure, as well as events
///
/// # Notes
/// - This is one of many components which are combined to form a task, other components are needed
/// to fuse them to a task.
///
/// - [`TaskFrame`] can be decorated with other task unit implementations to expand the behavior, such
///     as adding retry mechanism via [`RetryFrame`], adding timeout via [`TimeoutFrame`]... etc.
///
///     ## Examples Of Frame Decorators
///     - **``RetriableTaskFrame<TimeoutTaskFrame<T>>``**: Execute task frame `T`, when the
///     task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
///     then finish, otherwise if it exceeds its maximum duration or if the task frame failed then
///     abort it and retry it again, repeating this process `N` times (can be controlled by the developer)
///     with a delay per retry (can be controlled by the developer) `d`
///
///     - **``FallbackTaskFrame<TimeoutTaskFrame<T1>, RetriableTaskFrame<T2>>``**: Execute task frame `T1`,
///     when the task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
///     then finish, otherwise if it either fails or it reaches its maximum duration then execute
///     task frame `T2` (as a fallback), try/retry executing this task frame for `N` times (can be
///     controlled by the developer) with a delay per retry of `d` (can be controlled by the developer),
///     regardless if it succeeds at some time or fails entirely, return the result back
#[async_trait]
pub trait TaskFrame: Send + Sync {
    /// The main execution logic of the task, it is meant as an internal method
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError>;

    fn on_start(&self) -> TaskStartEvent;
    fn on_end(&self) -> TaskEndEvent;
}

#[async_trait]
impl<F: TaskFrame + ?Sized> TaskFrame for Arc<F> {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        self.as_ref().execute(metadata, emitter).await
    }

    fn on_start(&self) -> TaskStartEvent {
        self.as_ref().on_start()
    }

    fn on_end(&self) -> TaskEndEvent {
        self.as_ref().on_end()
    }
}

/// [`TaskFrameBuilder`] acts more as a utility rather than a full feature, it allows to construct
/// the default implemented task frames with a more builder syntax
///
/// # Example
/// ```ignore
/// use std::num::NonZeroU32;
/// use std::time::Duration;
/// use chronolog_core::task::{ExecutionTaskFrame, TaskFrameBuilder};
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
/// // mitigate this with the newtype pattern)
/// ```
pub struct TaskFrameBuilder<T: TaskFrame>(T);

impl<T: TaskFrame> TaskFrameBuilder<T> {
    pub fn new(frame: T) -> Self {
        Self(frame)
    }

    pub fn with_instant_retry(
        self,
        retries: NonZeroU32,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(RetriableTaskFrame::new_instant(self.0, retries))
    }

    pub fn with_retry(
        self,
        retries: NonZeroU32,
        delay: Duration,
    ) -> TaskFrameBuilder<RetriableTaskFrame<T>> {
        TaskFrameBuilder(RetriableTaskFrame::new(self.0, retries, delay))
    }

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
        predicate: impl FramePredicateFunc + 'static,
    ) -> TaskFrameBuilder<ConditionalFrame<T>> {
        let condition: ConditionalFrame<T> = ConditionalFrame::<T>::builder()
            .predicate(predicate)
            .task(self.0)
            .error_on_false(false)
            .build();
        TaskFrameBuilder(condition)
    }

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

    pub fn build(self) -> T {
        self.0
    }
}
