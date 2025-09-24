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

use crate::scheduling_strats::*;
use std::fmt::Debug;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;

/*
    Quite a similar situation to ConditionalTaskFrame, tho this time I can save one builder and a
    from trait implementation, reducing the code and making it more maintainable
*/

/// Task Config is simply used as a builder to construct [`Task`], <br />
/// it isn't meant to be used by itself, you may refer to [`Task::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = Task))]
pub struct TaskConfig {
    /// The [`TaskMetadata`], it is the <u>**State**</u> of the task and is a reactive container, allowing
    /// the outside parties to listen to fields changing via [`ObserverField`], making it a very powerful
    /// system. Multiple listeners can be attached per field. For triggering an action by changing
    /// multiple fields, multiple listeners will need to be attached per field, and these listeners
    /// will need their own state and based on it either do nothing or execute a specific logic
    ///
    /// # Default Value
    /// By default, the value uses [`TaskMetadata`], which is an implementation of [`TaskMetadata],
    /// hosting the minimum number of fields that define a metadata container
    ///
    /// # See Also
    /// - [`TaskMetadata`]
    /// - [`ObserverField`]
    #[builder(default = Arc::new(TaskMetadata::new()))]
    metadata: Arc<TaskMetadata>,

    /// [`TaskPriority`] is a mechanism for <u>**Prioritizing Important Tasks**</u>, the greater the importance,
    /// the more ChronoGrapher ensures to execute exactly at the time when under heavy workflow and
    /// generally prioritize it over others. Priorities are separated to multiple tiers which are further
    /// explained in [`TaskPriority`] on what each variant serves
    ///
    /// # Default Value
    /// By default, every task is [`TaskPriority::MODERATE`]
    ///
    /// # See Also
    /// - [`TaskPriority`]
    #[builder(default = TaskPriority::MODERATE)]
    priority: TaskPriority,

    /// [`TaskFrame`] is the <u>**Main Logic Part Of The Task**</u>, this is where the logic lives in.
    /// It is an essential part of the system (as without it, a task is useless), more information
    /// can be viewed on the [`TaskFrame`] documentation on what its capabilities truly are
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`ExecutionTaskFrame`]
    /// - [`RetriableTaskFrame`]
    /// - [`TimeoutTaskFrame`]
    /// - [`FallbackTaskFrame`]
    /// - [`DependencyTaskFrame`]
    #[builder(setter(transform = |s: impl TaskFrame + 'static| Arc::new(s) as Arc<dyn TaskFrame>))]
    frame: Arc<dyn TaskFrame>,

    /// [`TaskSchedule`] defines <u>**When The Task Should Run**</u>, when a scheduler requests a
    /// ``reschedule``, the [`TaskSchedule`] computes the next point of time to execute the task, there
    /// are various default implementations which can be seen. This is also an essential part
    /// (as without it, the scheduler never knows when to run a task), for more information check the
    /// [`TaskSchedule`] documentation
    ///
    /// # See Also
    /// - [`TaskSchedule`]
    /// - [`Scheduler`]
    /// - [`TaskScheduleCalendar`]
    /// - [`TaskScheduleCron`]
    /// - [`TaskScheduleImmediate`]
    /// - [`TaskScheduleInterval`]
    #[builder(setter(transform = |s: impl TaskSchedule + 'static| Arc::new(s) as Arc<dyn TaskSchedule>))]
    schedule: Arc<dyn TaskSchedule>,

    /// [`TaskErrorHandler`] is the part which <u>**Handles Gracefully Any Errors / Failures That Happen
    /// Throughout The Task's Lifecycle**</u>. It has access to the error instance and is mostly meant to
    /// be used in case of cleanups, closing database connections... etc.
    ///
    /// # Default Value
    /// By default, every task has the error handler [`SilentTaskErrorHandler`], which silently ignores
    /// any error (i.e. Doesn't gracefully handle it), for any demos this is fine, but for any application
    /// **THIS SHOULD BE AVOIDED AND INSTEAD IDIOMATICALLY HANDLE THE ERROR YOURSELF**
    ///
    /// # See Also
    /// - [`TaskErrorHandler`]
    /// - [`SilentTaskErrorHandler`]
    /// - [`PanicTaskErrorHandler`]
    #[builder(
        default = Arc::new(SilentTaskErrorHandler),
        setter(transform = |s: impl TaskErrorHandler + 'static| Arc::new(s) as Arc<dyn TaskErrorHandler>)
    )]
    error_handler: Arc<dyn TaskErrorHandler>,

    /// [`ScheduleStrategy`] is the part where <u>**It Controls How The Rescheduling Happens And How The Same
    /// Tasks Overlap With Each Other**</u>. There are various implementations, each suited for their own use
    /// case which are documented thoroughly on [`ScheduleStrategy`]
    ///
    /// # Default Value
    /// By default, every task uses the [`SequentialSchedulingPolicy`], which executes a task first
    /// then reschedules that task. This means no matter what, there will **NEVER** be a scenario
    /// where the same task overlaps itself
    ///
    /// # See Also
    /// - [`ScheduleStrategy`]
    /// - [`SequentialSchedulingPolicy`]
    /// - [`ConcurrentSchedulingPolicy`]
    /// - [`CancelPreviousSchedulingPolicy`]
    /// - [`CancelCurrentSchedulingPolicy`]
    #[builder(
        default = Arc::new(SequentialSchedulingPolicy),
        setter(transform = |s: impl ScheduleStrategy + 'static| Arc::new(s) as Arc<dyn ScheduleStrategy>)
    )]
    schedule_strategy: Arc<dyn ScheduleStrategy>,

    /// This part is mostly for debugging, more specifically to identify tasks, you can
    /// give it your own string (ideally it should be unique)
    ///
    /// # Default Value
    /// By default, every task has a generated UUID string, this may complicate things
    /// for debugging, as such. It is suggested to **always** fill this field with a unique name
    /// to save yourself from the time wasted and confusion
    #[builder(default = Uuid::new_v4().to_string())]
    debug_label: String,

    /// This part controls the maximum number of runs a task is allowed,
    /// before being canceled from the scheduler
    ///
    /// # Default Value
    /// By default, every task can run an infinite number of times (i.e. Has as value None), this
    /// may sometimes be an undesirable behavior to run a task forever, as such this is why this
    /// parameter exists
    #[builder(default = None, setter(strip_option))]
    max_runs: Option<NonZeroU64>,
}

impl From<TaskConfig> for Task {
    fn from(config: TaskConfig) -> Self {
        Task {
            metadata: config.metadata,
            frame: config.frame,
            schedule: config.schedule,
            error_handler: config.error_handler,
            overlap_policy: config.schedule_strategy,
            priority: config.priority,
            runs: AtomicU64::new(0),
            debug_label: config.debug_label,
            max_runs: config.max_runs,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
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
///   was executed... etc. The task metadata is also reactive, as most fields are [`ObserverField`],
///   allowing the developers to listen in various fields for changes made to them. Any outside parties
///   can access it via using [`Task::metadata`]
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
/// - **[`ScheduleStrategy`]** Defines how the scheduler should handle the rescheduling of a task and
///   how it handles task overlapping behavior. By default, (the parameter is optional to define),
///   it runs sequentially. i.e. The task only reschedules once it is fully finished
/// ---
///
/// In order to actually use the task, the developer must register it in a [`Scheduler`], could be
/// the default implementation of the scheduler or a custom-made, regardless, the task object is useless
/// without registration of it
///
/// # Trait Implementation(s)
/// [`Task`] implements debug, which is displayed in the form of a tuple struct containing debug
/// label. By default, it is a random UUID, which may be gibberish when debugging, as such it is
/// advised to provide a name for the task to identify it easily
///
/// # See Also
/// - [`TaskFrame`]
/// - [`TaskMetadata`]
/// - [`ExposedTaskMetadata`]
/// - [`Scheduler`]
/// - [`TaskEvent`]
/// - [`TaskSchedule`]
/// - [`ScheduleStrategy`]
/// - [`TaskErrorHandler`]
pub struct Task {
    pub(crate) metadata: Arc<TaskMetadata>,
    pub(crate) frame: Arc<dyn TaskFrame>,
    pub(crate) schedule: Arc<dyn TaskSchedule>,
    pub(crate) error_handler: Arc<dyn TaskErrorHandler>,
    pub(crate) overlap_policy: Arc<dyn ScheduleStrategy>,
    pub(crate) priority: TaskPriority,
    pub(crate) runs: AtomicU64,
    pub(crate) debug_label: String,
    pub(crate) max_runs: Option<NonZeroU64>,
    pub on_start: TaskStartEvent,
    pub on_end: TaskEndEvent,
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Task").field(&self.debug_label).finish()
    }
}

impl Task {
    /// A simple constructor that creates a simple task from a task schedule and a task frame.
    /// Mostly used as a convenient method for simple enough tasks that don't need any of the other
    /// composite parts. Otherwise, the [`Task::builder`] may be preferred over.
    ///
    /// # Arguments
    /// - **schedule** The task schedule, it is used for computing when the task should run.
    /// - **task** The task frame, it is the logic part of the task.
    ///
    /// # Returns
    /// The [`Task`] built from these 2 arguments, with the remaining fields being default values
    ///
    /// # Example
    /// ```ignore
    /// use chronographer_core::task::Task;
    /// use chronographer_core::schedule::TaskScheduleImmediate;
    /// use chronographer_core::task::frames::ExecutionTaskFrame;
    ///
    /// Task::define(
    ///     TaskScheduleImmediate,
    ///     ExecutionTaskFrame::new(|_| async {
    ///         todo!()
    ///     })
    /// );
    /// ```
    pub fn define(schedule: impl TaskSchedule + 'static, task: impl TaskFrame + 'static) -> Self {
        Self {
            frame: Arc::new(task),
            metadata: Arc::new(TaskMetadata::new()),
            schedule: Arc::new(schedule),
            error_handler: Arc::new(SilentTaskErrorHandler),
            overlap_policy: Arc::new(SequentialSchedulingPolicy),
            priority: TaskPriority::MODERATE,
            runs: AtomicU64::new(0),
            debug_label: Uuid::new_v4().to_string(),
            max_runs: None,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
        }
    }

    /// Creates a task builder without an extension point required, this is mostly a
    /// convenience method and is identical to doing:
    /// ```ignore
    /// use chronographer_core::task::Task;
    ///
    /// Task::extend_builder()
    ///     .extension(())
    /// ```
    ///
    /// # Example
    /// ```ignore
    /// use chronographer_core::task::{
    ///     ExecutionTaskFrame, PanicTaskErrorHandler,
    ///     Task, TaskScheduleImmediate
    /// };
    ///
    /// Task::builder()
    ///     .schedule(TaskScheduleImmediate)
    ///     .frame(ExecutionTaskFrame::new(|_| async {
    ///         todo!()
    ///     }))
    ///     .error_handler(PanicTaskErrorHandler)
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`Task::define`]
    /// - [`Task::extend_builder`]
    /// - [`TaskExtension`]
    pub fn builder() -> TaskConfigBuilder {
        TaskConfig::builder()
    }

    /// Runs the task, handling any metadata throughout by itself as well as calling events
    /// the error handler. This method can only be used by parts which have access to [`TaskEventEmitter`],
    /// such as [`Scheduler`], and mostly is an internal one (even if exposed for public use)
    ///
    /// # See Also
    /// - [`TaskEventEmitter`]
    /// - [`Scheduler`]
    pub async fn run(&self, emitter: Arc<TaskEventEmitter>) -> Result<(), TaskError> {
        self.runs.fetch_add(1, Ordering::Relaxed);
        emitter
            .emit(self.metadata(), self.on_start.clone(), ())
            .await;
        let result = self
            .frame()
            .execute(TaskContext::new(self, emitter.clone()))
            .await;
        let err = result.clone().err();

        emitter
            .emit(self.metadata(), self.on_end.clone(), err.clone())
            .await;

        if let Some(error) = err {
            let error_ctx = TaskErrorContext {
                error,
                metadata: self.metadata(),
            };
            self.error_handler().on_error(Arc::new(error_ctx)).await;
        }

        result
    }

    /// Gets the exposed metadata (immutable) for outside parties
    pub fn metadata(&self) -> Arc<TaskMetadata> {
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
    pub fn schedule_strategy(&self) -> Arc<dyn ScheduleStrategy> {
        self.overlap_policy.clone()
    }

    /// Gets the priority of a task
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    /// Gets the number of times the task has run
    pub fn runs(&self) -> u64 {
        self.runs.load(Ordering::Relaxed)
    }

    /// Gets the maximum number of times the task can run (``None`` for infinite times)
    pub fn max_runs(&self) -> &Option<NonZeroU64> {
        &self.max_runs
    }
}
