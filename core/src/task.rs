#[allow(missing_docs)]
pub mod dependency;  // skipcq: RS-D1001

#[allow(missing_docs)]
pub mod frames;  // skipcq: RS-D1001

#[allow(missing_docs)]
pub mod priority;  // skipcq: RS-D1001

#[allow(missing_docs)]
pub mod frame_builder;  // skipcq: RS-D1001

#[allow(missing_docs)]
pub mod hooks;  // skipcq: RS-D1001

pub use crate::schedule::*;
pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use priority::*;

use crate::scheduling_strats::*;
use dashmap::DashMap;
use std::fmt::Debug;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;

/// [`TaskConfig`] is simply used as a builder to construct [`Task`], <br />
/// it isn't meant to be used by itself, you may refer to [`Task::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = Task))]
#[builder(mutators(
    fn hook<E: TaskHookEvent>(&mut self, hook: Arc<dyn TaskHook<E>>){
        self.hooks.attach::<E>(hook);
    }
))]
pub struct TaskConfig {
    #[builder(via_mutators(init = TaskHookContainer(DashMap::default())))]
    hooks: TaskHookContainer,

    /// [`TaskPriority`] is a mechanism for <u>**Prioritizing Important Tasks**</u>, the greater the importance,
    /// the more ChronoGrapher ensures to execute exactly at the time when under heavy workflow and
    /// generally prioritize it over others. Priorities are separated to multiple tiers which are further
    /// explained in [`TaskPriority`] on what each variant serves
    ///
    /// # Default Value
    /// By default, every task is [`TaskPriority::MODERATE`]
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskPriority`]
    #[builder(default = TaskPriority::MODERATE)]
    priority: TaskPriority,

    /// [`TaskFrame`] is the <u>**Main Logic Part Of The Task**</u>, this is where the logic lives in.
    /// It is an essential part of the system (as without it, a task is useless), more information
    /// can be viewed on the [`TaskFrame`] documentation on what its capabilities truly are
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
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
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
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

    /// [`ScheduleStrategy`] is the part where <u>**It Controls How The Rescheduling Happens And How The Same
    /// Tasks Overlap With Each Other**</u>. There are various implementations, each suited for their own use
    /// case which are documented thoroughly on [`ScheduleStrategy`]
    ///
    /// # Default Value
    /// By default, every task uses the [`SequentialSchedulingPolicy`], which executes a task first
    /// then reschedules that task. This means no matter what, there will **NEVER** be a scenario
    /// where the same task overlaps itself
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
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
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    #[builder(default = Uuid::new_v4().to_string())]
    debug_label: String,

    /// This part controls the maximum number of runs a task is allowed,
    /// before being canceled from the scheduler
    ///
    /// # Default Value
    /// By default, every task can run an infinite number of times (i.e. Has as value None), this
    /// may sometimes be an undesirable behavior to run a task forever, as such this is why this
    /// parameter exists
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    #[builder(default = None, setter(strip_option))]
    max_runs: Option<NonZeroU64>,
}

impl From<TaskConfig> for Task {
    fn from(config: TaskConfig) -> Self {
        Task {
            frame: config.frame,
            schedule: config.schedule,
            hooks: Arc::new(config.hooks),
            overlap_policy: config.schedule_strategy,
            priority: config.priority,
            runs: Arc::new(AtomicU64::new(0)),
            debug_label: config.debug_label,
            max_runs: config.max_runs,
            id: Uuid::new_v4(),
        }
    }
}

/// [`Task`] is one of the core components of ChronoGrapher, it is a composite, and made of several parts,
/// giving it massive flexibility in terms of customization.
///
/// # Implementation Detail(s)
/// Task is not just one entity, rather it has many moving parts, many composites, the important
/// ones are:
///
/// - **[`TaskMetadata`]** The <u>Local Task State</u>, it is a reactive container, allowing
///   the ability to listen to various incoming field changes, it can be modified from any point, it also
///   allows tracking of dynamic fields, in addition outside parties can also use and modify it via
///   [`Task::metadata`]
///
/// - **[`TaskFrame`]** The <u>What</u> of the task, the logic part of the task. When executed, task
///   frames get a task context which hosts all the information needed, including an event emitter,
///   metadata, debug label... etc. the emitter can be used to emit their own events. Task frames
///   can be decorated with other task frames to form a chain of task frames, allowing for complex
///   logic (and policy logic) to be injected to the task without manual writing. There are various
///   implementations of task frame and the task frame can be accessed via [`Task::frame`]
///
/// - **[`TaskSchedule`]** The <u>When</u> will the task execute, it is used for calculating the next
///   time to invoke this task. This part is useful to the scheduler mostly, tho outside parties can
///   also use it via [`Task::schedule`]
///
/// - **[`TaskErrorHandler`]** An error handler for the task, in case things go south. By default,
///   it doesn't need to be supplied, and it will silently ignore the error, tho ideally in most cases
///   it should be supplied for fine-grain error handling. When invoked, the task error handler gets
///   a context object hosting the exposed metadata and the error. It is meant to return nothing, just
///   handle the error the task gave away, outside parties can access this via [`Task::error_handler`]
///
/// - **[`ScheduleStrategy`]** Defines how the scheduler should handle the rescheduling of a task and
///   how it handles task overlapping behavior. By default, (the parameter is optional to define),
///   it runs sequentially. i.e. The task only reschedules once it is fully finished, outside parties
///   can access this via [`Task::schedule_strategy`]
///
/// Other minor parts include a debug label, maximum runs... etc. In order to actually use the task,
/// the developer must register it in a [`Scheduler`], could be the default implementation of the
/// scheduler or a custom-made, regardless, the task object is useless without registration of it
///
/// # Constructor(s)
/// There are 2 ways when it comes to creating a [`Task`]. The former is [`Task::define`] which
/// is used for defining simple tasks that only need a frame and a schedule (the important parts)
/// and acts as a convenience method, while the latter is [`Task::builder`] which creates a builder,
/// allowing more customization over individual fields
///
/// [`Task`] cannot be constructed like a typical ``struct`` due to the fact it contains
/// some information that is meant to have a default value and not have the initial value
/// controlled by the user
///
/// # Trait Implementation(s)
/// [`Task`] implements debug, which is displayed in the form of a tuple struct containing debug
/// label. By default, it is a random UUID, which may be gibberish when debugging, as such it is
/// advised to provide a name for the task to identify it easily. [`Task`] also implements clone
///
/// # Cloning Semantics
/// When cloning a [`Task`]
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
    frame: Arc<dyn TaskFrame>,
    schedule: Arc<dyn TaskSchedule>,
    overlap_policy: Arc<dyn ScheduleStrategy>,
    priority: TaskPriority,
    runs: Arc<AtomicU64>,
    debug_label: String,
    max_runs: Option<NonZeroU64>,
    id: Uuid,
    hooks: Arc<TaskHookContainer>,
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Task").field(&self.debug_label).finish()
    }
}

impl Task {
    /// A simple constructor that creates a simple [`Task`] from a task schedule and a task frame.
    /// Mostly used as a convenient method for simple enough tasks that don't need any of the other
    /// composite parts. Otherwise, the [`Task::builder`] may be preferred over.
    ///
    /// # Arguments
    /// - **schedule** The [`TaskSchedule`], it is used for computing when the task should run.
    /// - **task** The [`TaskFrame`], it is the logic part of the task.
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
    ///         unimplemented!()
    ///     })
    /// );
    ///
    /// ```
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`Task::builder`]
    /// - [`TaskFrame`]
    /// - [`TaskSchedule`]
    pub fn define(schedule: impl TaskSchedule + 'static, task: impl TaskFrame + 'static) -> Self {
        Self {
            frame: Arc::new(task),
            schedule: Arc::new(schedule),
            hooks: Arc::new(TaskHookContainer(DashMap::default())),
            overlap_policy: Arc::new(SequentialSchedulingPolicy),
            priority: TaskPriority::MODERATE,
            runs: Arc::new(AtomicU64::new(0)),
            debug_label: Uuid::new_v4().to_string(),
            max_runs: None,
            id: Uuid::new_v4(),
        }
    }

    /// Creates a [`Task`] builder used for more customization on the fields. For convenience,
    /// if your task only consists of [`TaskSchedule`] and [`TaskFrame`] and you don't plan
    /// to modify any fields apart from the defaults, then [`Task::define`] does a better job
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
    ///         unimplemented!()
    ///     }))
    ///     .error_handler(PanicTaskErrorHandler)
    ///     .build();
    /// ```
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`Task::define`]
    /// - [`TaskSchedule`]
    /// - [`TaskFrame`]
    pub fn builder() -> TaskConfigBuilder {
        TaskConfig::builder()
    }

    /// Runs the task, handling any data throughout by itself as well as calling events
    /// the error handler. This method can only be used by parts which have access to [`TaskEventEmitter`],
    /// such as [`Scheduler`], and mostly is an internal one (even if exposed for public use)
    ///
    /// # See Also
    /// - [`TaskEventEmitter`]
    /// - [`Scheduler`]
    pub async fn run(&self) -> Result<(), TaskError> {
        let ctx = TaskContext::new(self);
        self.runs.fetch_add(1, Ordering::Relaxed);
        ctx.emit::<OnTaskStart>(&ctx.clone()).await;
        let result = self.frame.execute(ctx.clone()).await;
        let err = result.clone().err();

        ctx.clone().emit::<OnTaskEnd>(&(ctx, err)).await;

        result
    }

    /// Gets the [`TaskFrame`] for outside parties
    pub fn frame(&self) -> Arc<dyn TaskFrame> {
        self.frame.clone()
    }

    /// Gets the [`TaskSchedule`] for outside parties
    pub fn schedule(&self) -> Arc<dyn TaskSchedule> {
        self.schedule.clone()
    }

    /// Gets the [`ScheduleStrategy`] for outside parties
    pub fn schedule_strategy(&self) -> Arc<dyn ScheduleStrategy> {
        self.overlap_policy.clone()
    }

    /// Gets the [`TaskPriority`] for a task
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

    /// Gets the ID associated with the [`Task`]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Gets the hooks container the [`Task`] has
    pub fn hooks(&self) -> Arc<TaskHookContainer> {
        self.hooks.clone()
    }

    pub async fn attach_hook<E: TaskHookEvent>(&self, hook: Arc<dyn TaskHook<E>>) {
        self.hooks.attach(hook).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        self.hooks.get::<E, T>()
    }

    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        self.hooks.detach::<E, T>().await;
    }
}
