use crate::errors::ChronographerErrors;
use crate::task::{
    ArcTaskEvent, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter, TaskFrame, TaskMetadata,
    TaskStartEvent,
};
use async_trait::async_trait;
use std::sync::Arc;
use typed_builder::TypedBuilder;

/*
    Sadly, I am forced to specify and handle 2 mostly identical
    builder types for ergonomic reasons, unless if someone suggests
    a better way of doing this, then it pretty much stays this way.

    Honestly, it's quite the code smell
*/

#[async_trait]
pub trait FramePredicateFunc: Send + Sync {
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> bool;
}

#[async_trait]
impl<F, Fut> FramePredicateFunc for F
where
    F: Fn(Arc<dyn TaskMetadata>) -> Fut + Send + Sync,
    Fut: Future<Output = bool> + Send,
{
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> bool {
        self(metadata).await
    }
}

#[async_trait]
impl<F: FramePredicateFunc + ?Sized> FramePredicateFunc for Arc<F> {
    async fn execute(&self, metadata: Arc<dyn TaskMetadata>) -> bool {
        self.as_ref().execute(metadata).await
    }
}

//noinspection DuplicatedCode
#[derive(TypedBuilder)]
#[builder(build_method(into = ConditionalFrame<T, T2>))]
pub struct FallbackConditionalFrameConfig<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    #[builder(setter(transform = |s: T2| Arc::new(s)))]
    fallback: Arc<T2>,

    #[builder(setter(transform = |s: T| Arc::new(s)))]
    task: Arc<T>,

    #[builder(setter(transform = |s: impl FramePredicateFunc + 'static| {
        Arc::new(s) as Arc<dyn FramePredicateFunc>
    }))]
    predicate: Arc<dyn FramePredicateFunc>,

    #[builder(default = false)]
    error_on_false: bool,
}

//noinspection DuplicatedCode
#[derive(TypedBuilder)]
#[builder(build_method(into = ConditionalFrame<T, ()>))]
pub struct NonFallbackConditionalFrameConfig<T: TaskFrame + Send + Sync + 'static> {
    #[builder(setter(transform = |s: T| Arc::new(s)))]
    task: Arc<T>,

    #[builder(setter(transform = |s: impl FramePredicateFunc + 'static| {
        Arc::new(s) as Arc<dyn FramePredicateFunc>
    }))]
    predicate: Arc<dyn FramePredicateFunc>,

    #[builder(default = false)]
    error_on_false: bool,
}

impl<T, T2> From<FallbackConditionalFrameConfig<T, T2>> for ConditionalFrame<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    fn from(config: FallbackConditionalFrameConfig<T, T2>) -> Self {
        ConditionalFrame {
            task: config.task,
            fallback: config.fallback,
            predicate: config.predicate,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            error_on_false: config.error_on_false,
            on_true: TaskEvent::new(),
            on_false: TaskEvent::new(),
        }
    }
}

impl<T: TaskFrame + 'static + Send + Sync> From<NonFallbackConditionalFrameConfig<T>>
    for ConditionalFrame<T>
{
    fn from(config: NonFallbackConditionalFrameConfig<T>) -> Self {
        ConditionalFrame {
            task: config.task,
            fallback: Arc::new(()),
            predicate: config.predicate,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            error_on_false: config.error_on_false,
            on_true: TaskEvent::new(),
            on_false: TaskEvent::new(),
        }
    }
}

/// Represents a **conditional task frame** which wraps a task frame and executes it depending on a
/// predicate function. This task frame type acts as a **wrapper node** within the task frame hierarchy,
/// facilitating a way to conditionally execute a task frame
///
/// A fallback can optionally be registered for the [`ConditionalFrame`] to allow the execution of another
/// task frame in case the predicate returns false (otherwise nothing is executed). The results from
/// the fallback will be returned if that is the case, otherwise the primary task may return
///
/// There is an important difference between [`ConditionalFrame`] and [`FallbackTaskFrame`], the
/// former relies on the predicate to determine which task frame to execute, while the latter relies
/// on whenever or not the primary task failed. In addition, [`ConditionalFrame`] **WILL NOT** execute
/// any task frame unless it has the boolean value, whereas [`FallbackTaskFrame`] **WILL** execute the
/// primary task always and potentially then the second task frame
///
/// # Events
/// For events, [`ConditionalFrame`] has only two events, those being ``on_true`` and ``on_false``,
/// they execute depending on the predicate and both host the target task frame which will be executed
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::conditionframe::ConditionalFrame;
/// use chronographer_core::task::Task;
///
/// let primary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // This is optional to specify
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let conditional_frame: ConditionalFrame<ExecutionTaskFrame<_>, ExecutionTaskFrame<_>> =
///     ConditionalFrame::builder()
///         .task(primary_frame)
///         .fallback(secondary_frame) // Remove this to not specify a fallback
///         .error_on_false(true) // Also an optional parameter, but can be useful in some cases
///         .predicate(|metadata| metadata.runs() % 2 == 0)
///         .build();
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(3.21), conditional_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct ConditionalFrame<T: 'static, T2: 'static = ()> {
    task: Arc<T>,
    fallback: Arc<T2>,
    predicate: Arc<dyn FramePredicateFunc>,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
    error_on_false: bool,
    pub on_true: ArcTaskEvent<Arc<T>>,
    pub on_false: ArcTaskEvent<Arc<T2>>,
}

impl<T, T2> ConditionalFrame<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    pub fn fallback_builder() -> FallbackConditionalFrameConfigBuilder<T, T2> {
        FallbackConditionalFrameConfig::builder()
    }
}

impl<T: TaskFrame + 'static + Send + Sync> ConditionalFrame<T> {
    pub fn builder() -> NonFallbackConditionalFrameConfigBuilder<T> {
        NonFallbackConditionalFrameConfig::builder()
    }
}

macro_rules! execute_func_impl {
    ($self: expr, $emitter: expr, $metadata: expr) => {
        let result = $self.predicate.execute($metadata.clone()).await;
        if result {
            $emitter
                .emit($metadata.clone(), $self.on_true.clone(), $self.task.clone())
                .await;
            return $self.task.execute($metadata.clone(), $emitter).await;
        }
        $emitter
            .emit(
                $metadata.clone(),
                $self.on_false.clone(),
                $self.fallback.clone(),
            )
            .await;
    };
}

macro_rules! define_events {
    () => {
        fn on_start(&self) -> TaskStartEvent {
            self.on_start.clone()
        }

        fn on_end(&self) -> TaskEndEvent {
            self.on_end.clone()
        }
    };
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for ConditionalFrame<T> {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        execute_func_impl!(self, emitter, metadata);
        if self.error_on_false {
            Err(Arc::new(ChronographerErrors::TaskConditionFail))
        } else {
            Ok(())
        }
    }

    define_events!();
}

#[async_trait]
impl<T: TaskFrame, F: TaskFrame> TaskFrame for ConditionalFrame<T, F> {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        execute_func_impl!(self, emitter, metadata);
        let result = self.fallback.execute(metadata, emitter).await;
        if self.error_on_false && result.is_ok() {
            return Err(Arc::new(ChronographerErrors::TaskConditionFail));
        }
        result
    }

    define_events!();
}
