pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod scheduling_strats; // skipcq: RS-D1001

pub mod schedule; // skipcq: RS-D1001

pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use schedule::*;
pub use scheduling_strats::*;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use dashmap::DashMap;
use std::fmt::Debug;
use std::sync::Arc;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub type ErasedTask = Task<dyn TaskFrame, dyn TaskSchedule, dyn ScheduleStrategy>;

/// [`TaskConfig`] is simply used as a builder to construct [`Task`], <br />
/// it isn't meant to be used by itself, you may refer to [`Task::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = Task<T1, T2, T3>))]
pub struct TaskConfig<T1: TaskFrame, T2: TaskSchedule, T3: ScheduleStrategy> {
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
    frame: T1,

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
    schedule: T2,

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
    schedule_strategy: T3,
}

impl<T1: TaskFrame, T2: TaskSchedule, T3: ScheduleStrategy> From<TaskConfig<T1, T2, T3>>
    for Task<T1, T2, T3>
{
    fn from(config: TaskConfig<T1, T2, T3>) -> Self {
        Task {
            frame: Arc::new(config.frame),
            schedule: Arc::new(config.schedule),
            hooks: Arc::new(TaskHookContainer(DashMap::default())),
            schedule_strategy: Arc::new(config.schedule_strategy),
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
/// - [`Scheduler`]
/// - [`TaskEvent`]
/// - [`TaskSchedule`]
/// - [`ScheduleStrategy`]
// #[derive(Serialize, Deserialize)]
pub struct Task<T1: ?Sized + 'static, T2: ?Sized + 'static, T3: ?Sized + 'static> {
    frame: Arc<T1>,
    schedule: Arc<T2>,
    schedule_strategy: Arc<T3>,
    id: Uuid,
    hooks: Arc<TaskHookContainer>,
}

impl<T1: TaskFrame, T2: TaskSchedule> Task<T1, T2, SequentialSchedulingPolicy> {
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
    pub fn simple(schedule: T2, frame: T1) -> Self {
        let id = Uuid::new_v4();
        Self {
            frame: Arc::new(frame),
            schedule: Arc::new(schedule),
            hooks: Arc::new(TaskHookContainer(DashMap::default())),
            schedule_strategy: Arc::new(SequentialSchedulingPolicy),
            id,
        }
    }
}

impl<T1: TaskFrame, T2: TaskSchedule, T3: ScheduleStrategy> Task<T1, T2, T3> {
    /// Gets the [`TaskFrame`] for outside parties
    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    /// Gets the [`TaskSchedule`] for outside parties
    pub fn schedule(&self) -> &T2 {
        &self.schedule
    }

    /// Gets the [`ScheduleStrategy`] for outside parties
    pub fn schedule_strategy(&self) -> &T3 {
        &self.schedule_strategy
    }
}

impl ErasedTask {
    /// Runs the task, handling any data throughout by itself as well as calling events
    /// the error handler. This method can only be used by parts which have access to [`TaskEventEmitter`],
    /// such as [`Scheduler`], and mostly is an internal one (even if exposed for public use)
    ///
    /// # See Also
    /// - [`TaskEventEmitter`]
    /// - [`Scheduler`]
    pub async fn run(&self) -> Result<(), TaskError> {
        let ctx = TaskContext::new(self);
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
        let result = self.frame.execute(&ctx).await;
        let err = result.clone().err();

        ctx.emit::<OnTaskEnd>(&err).await;

        result
    }

    /// Gets the [`TaskFrame`] for outside parties
    pub fn frame(&self) -> &dyn TaskFrame {
        &self.frame
    }

    /// Gets the [`TaskSchedule`] for outside parties
    pub fn schedule(&self) -> &dyn TaskSchedule {
        &self.schedule
    }

    /// Gets the [`ScheduleStrategy`] for outside parties
    pub fn schedule_strategy(&self) -> &dyn ScheduleStrategy {
        &self.schedule_strategy
    }
}

impl<T1: ?Sized, T2: ?Sized, T3: ?Sized> Task<T1, T2, T3> {
    /// Gets the ID associated with the [`Task`]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Gets the hooks container the [`Task`] has
    pub fn hooks(&self) -> &TaskHookContainer {
        &self.hooks
    }
}

impl<T1: TaskFrame, T2: TaskSchedule, T3: ScheduleStrategy> Task<T1, T2, T3> {
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
    pub fn builder() -> TaskConfigBuilder<T1, T2, T3> {
        TaskConfig::builder()
    }

    pub fn as_erased(&self) -> ErasedTask {
        ErasedTask {
            frame: self.frame.clone(),
            schedule: self.schedule.clone(),
            schedule_strategy: self.schedule_strategy.clone(),
            id: self.id,
            hooks: self.hooks.clone(),
        }
    }

    pub async fn attach_hook<E: TaskHookEvent>(&self, hook: Arc<impl TaskHook<E>>) {
        let ctx = TaskContext::new(&self.as_erased());
        self.hooks.attach(&ctx, hook).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        self.hooks.get::<E, T>()
    }

    pub async fn emit_hook_event<E: TaskHookEvent>(&self, payload: &E::Payload) {
        let ctx = TaskContext::new(&self.as_erased());
        self.hooks.emit::<E>(&ctx, payload).await;
    }

    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        let ctx = TaskContext::new(&self.as_erased());
        self.hooks.detach::<E, T>(&ctx).await;
    }
}
