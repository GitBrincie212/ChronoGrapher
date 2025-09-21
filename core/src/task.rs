pub mod dependency;
pub mod error_handler;
pub mod events;
pub mod frames;
pub mod metadata;
pub mod priority;

pub use crate::schedule::*;
pub use error_handler::*;
pub use events::*;
pub use frames::*;
pub use metadata::*;
pub use priority::*;

use crate::scheduling_strats::{ScheduleStrategy, SequentialSchedulingPolicy};
use std::fmt::Debug;
use std::ops::Add;
use std::sync::Arc;
use typed_builder::TypedBuilder;

/*
    Quite a similar situation to ConditionalTaskFrame, tho this time I can save one builder and a
    from trait implementation, reducing the code and making it more maintainable
*/

#[derive(TypedBuilder)]
#[builder(build_method(into = Task<E>))]
pub struct TaskConfig<E: TaskExtension> {
    extension: E,

    #[builder(
        default = Arc::new(DefaultTaskMetadata::new()),
        setter(transform = |s: impl TaskMetadata + 'static| Arc::new(s) as Arc<dyn TaskMetadata>)
    )]
    metadata: Arc<dyn TaskMetadata>,

    #[builder(default = TaskPriority::MODERATE)]
    priority: TaskPriority,

    #[builder(setter(transform = |s: impl TaskFrame + 'static| Arc::new(s) as Arc<dyn TaskFrame>))]
    frame: Arc<dyn TaskFrame>,

    #[builder(setter(transform = |s: impl TaskSchedule + 'static| Arc::new(s) as Arc<dyn TaskSchedule>))]
    schedule: Arc<dyn TaskSchedule>,

    #[builder(
        default = Arc::new(SilentTaskErrorHandler),
        setter(transform = |s: impl TaskErrorHandler + 'static| Arc::new(s) as Arc<dyn TaskErrorHandler>)
    )]
    error_handler: Arc<dyn TaskErrorHandler>,

    #[builder(
        default = Arc::new(SequentialSchedulingPolicy),
        setter(transform = |s: impl ScheduleStrategy + 'static| Arc::new(s) as Arc<dyn ScheduleStrategy>)
    )]
    overlap_policy: Arc<dyn ScheduleStrategy>,
}

impl<E: TaskExtension> From<TaskConfig<E>> for Task<E> {
    fn from(config: TaskConfig<E>) -> Self {
        Task {
            metadata: config.metadata,
            frame: config.frame,
            schedule: config.schedule,
            error_handler: config.error_handler,
            overlap_policy: config.overlap_policy,
            priority: config.priority,
            extension: Arc::new(config.extension),
        }
    }
}

/// [`Task`] is one of the core components of ChronoGrapher, it is a composite, and made of several parts,
/// giving it massive flexibility in terms of customization.
///
/// # Task Composite Parts
///
/// - **[`TaskMetadata`]** The <u>State</u>, by default (the parameter is optional to define)
///   it contains information such as the run-count, the maximum runs allowed, the last time the task
///   was executed... etc. The task metadata can be exposed in the form of [`ExposedTaskMetadata`],
///   giving an immutable version of it, typically this metadata is exposed to the task frame, the error
///   handler and this exposed version can be used outside via [`Task::metadata`]
///
/// - **[`TaskFrame`]** The <u>What</u> of the task, the logic part of the task. When executed, task
///   frames get the exposed metadata and an event emitter for task events (lifecycle or local events,
///   see [`TaskEvent`] for more context), the emitter can be used to emit their own events. Task frames
///   can be decorated with other task frames to form a chain of task frames, allowing for complex
///   logic (and policy logic) to be injected to the task without manual writing. There are various
///   implementations of task frane and the task frame can be accessed via [`Task::frame`]
///
/// - **[`TaskSchedule`]** The <u>When</u> will the task execute, it is used for calculating the next
///   time to invoke this task. This part is useful to the scheduler mostly, tho outside parties can
///   also use it via [`Task::schedule`]
///
/// - **[`TaskErrorHandler`]** An error handler for the task, in case things go south. By default,
///   it doesn't need to be supplied, and it will silently ignore the error, tho ideally in most cases
///   it should be supplied for fine-grain error handling. When invoked, the task error handler gets
///   a context object hosting the exposed metadata and the error. It is meant to return nothing, just
///   handle the error the task gave away
///
/// - **[`ScheduleStrategy`]** Defines how it is scheduled and how it handles task overlapping
///   behavior. By default, (the parameter is optional to define), it runs sequentially. i.e. The task
///   only reschedules once it is fully finished
/// ---
///
/// In order to actually use the task, the developer must register it in a [`Scheduler`], could be
/// the default implementation of the scheduler or a custom-made, regardless, the task object is useless
/// without registration of it
///
/// # See
/// - [`TaskFrame`]
/// - [`TaskMetadata`]
/// - [`ExposedTaskMetadata`]
/// - [`Scheduler`]
/// - [`TaskEvent`]
/// - [`TaskSchedule`]
/// - [`ScheduleStrategy`]
/// - [`TaskErrorHandler`]
#[derive(Clone)]
pub struct Task<E: TaskExtension = ()> {
    pub(crate) metadata: Arc<dyn TaskMetadata>,
    pub(crate) frame: Arc<dyn TaskFrame>,
    pub(crate) schedule: Arc<dyn TaskSchedule>,
    pub(crate) error_handler: Arc<dyn TaskErrorHandler>,
    pub(crate) overlap_policy: Arc<dyn ScheduleStrategy>,
    pub(crate) priority: TaskPriority,
    pub(crate) extension: Arc<E>,
}

impl Debug for Task<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Task")
            .field(&self.metadata.debug_label())
            .finish()
    }
}

/// [`TaskExtension`] is a blanket trait, providing a way to add on more composite types to the
/// task, without having to deal with metadata management while getting guaranteed type safety.
/// By default, a [`Task`] does not require a task extension, as such this is mostly for third
/// party integrations
pub trait TaskExtension: Send + Sync {}
impl TaskExtension for () {}

type DefaultExtensiveBuilder = TaskConfigBuilder<(), (((),), (), (), (), (), (), ())>;

impl Task {
    /// Creates a simple task from a schedule and an interval. Mostly used as a convenient method
    /// for simple enough tasks that don't need any of the other composite parts
    pub fn define(schedule: impl TaskSchedule + 'static, task: impl TaskFrame + 'static) -> Self {
        Self {
            frame: Arc::new(task),
            metadata: Arc::new(DefaultTaskMetadata::new()),
            schedule: Arc::new(schedule),
            error_handler: Arc::new(SilentTaskErrorHandler),
            overlap_policy: Arc::new(SequentialSchedulingPolicy),
            priority: TaskPriority::MODERATE,
            extension: Arc::new(()),
        }
    }

    /// Creates a task builder without an extension point required, this is mostly a
    /// convenience method and is identical to:
    /// ```ignore
    /// # use chronographer_core::task::Task;
    ///
    /// Task::extend_builder()
    ///     .extension(())
    /// ```
    pub fn builder() -> DefaultExtensiveBuilder {
        TaskConfig::builder().extension(())
    }
}

impl<E: TaskExtension> Task<E> {
    /// Creates a task builder with a required extension point of type `E` [`TaskExtension`]
    pub fn extend_builder() -> TaskConfigBuilder<E> {
        TaskConfig::builder()
    }
}

impl<E: TaskExtension> Task<E> {
    pub async fn run(&self, emitter: Arc<TaskEventEmitter>) -> Result<(), TaskError> {
        self.metadata
            .runs()
            .update_internal(self.metadata.runs().get().add(1u64));
        emitter
            .emit(self.metadata(), self.frame().on_start(), ())
            .await;
        let result = self
            .frame()
            .execute(self.metadata.clone(), emitter.clone())
            .await;
        let err = result.clone().err();

        emitter
            .emit(self.metadata(), self.frame().on_end(), err.clone())
            .await;

        if let Some(error) = err {
            let error_ctx = TaskErrorContext {
                error,
                metadata: self.metadata.clone(),
            };
            self.error_handler().on_error(error_ctx).await;
        }

        result
    }

    /// Gets the exposed metadata (immutable) for outside parties
    pub fn metadata(&self) -> Arc<dyn TaskMetadata> {
        self.metadata.clone()
    }

    /// Gets the frame for outside parties
    pub fn frame(&self) -> Arc<dyn TaskFrame> {
        self.frame.clone()
    }

    /// Gets the schedule for outside parties
    pub fn schedule(&self) -> Arc<dyn TaskSchedule> {
        self.schedule.clone()
    }

    /// Gets the error handler for outside parties
    pub fn error_handler(&self) -> Arc<dyn TaskErrorHandler> {
        self.error_handler.clone()
    }

    /// Gets the overlapping policy for outside parties
    pub fn overlap_policy(&self) -> Arc<dyn ScheduleStrategy> {
        self.overlap_policy.clone()
    }

    /// Gets the extension trait attached to the task
    pub fn extension(&self) -> Arc<E> {
        self.extension.clone()
    }

    /// Gets the priority of a task
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
}
