use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{Debug, OnTaskEnd, ScheduleStrategy, TaskFrame, TaskHook, TaskHookContext, TaskHookEvent, TaskSchedule};
use crate::task::{Task, TaskError};
use async_trait::async_trait;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use typed_builder::TypedBuilder;

type IncompleteTaskDependencyConfig<T1, T2, T3> =
    TaskDependencyConfigBuilder<T1, T2, T3, ((&'static Task<T1, T2, T3>,), (), ())>;

/// [`TaskResolvent`] acts as a policy dictating how to manage the counting
/// of relevant runs towards [`TaskDependency`], the [`TaskDependency`] has
/// in store a caching mechanism to provide efficiency
///
/// # Required Method(s)
/// When implementing the trait [`TaskResolvent`], one has to supply an implementation
/// for the method [`TaskResolvent::should_count`] which is where the logic lives for
/// counting a task's run as relevant for [`TaskDependency`]
///
/// # Trait Implementation(s)
/// there are 3 core implementations of this [`TaskResolvent`] trait which are specifically:
/// - [`TaskResolveSuccessOnly`] counts only successful runs
/// - [`TaskResolveFailuresOnly`] counts only failure runs
/// - [`TaskResolveIdentity`] counts both successful and failure runs,
///
/// by default [`TaskDependency`] uses [`TaskResolveSuccessOnly`] (which of course can be overridden)
///
/// # See Also
/// - [`TaskDependency`]
/// - [`TaskResolveSuccessOnly`]
/// - [`TaskResolveFailuresOnly`]
/// - [`TaskResolveIdentity`]
#[async_trait]
pub trait TaskResolvent: Send + Sync {
    /// This is the main logic part that counts if a specific run
    /// is relevant to [`TaskDependency`] or not
    ///
    /// # Argument(s)
    /// This method accepts two arguments, that is an optional error
    /// of the [`Task`] represented as ``result`` as well as the
    /// [`TaskContext`] via ``ctx`` (without the event emission part, i.e. It
    /// is a restricted)
    ///
    /// # Returns
    /// A boolean value indicating that the run counts as relevant
    /// in [`TaskDependency`]
    ///
    /// # See Also
    /// - [`TaskDependency`]
    /// - [`TaskContext`]
    /// - [`TaskResolvent`]
    async fn should_count(&self, ctx: &TaskHookContext, result: Option<TaskError>) -> bool;
}

macro_rules! implement_core_resolvent {
    ($name: ident, $uuid: expr, $code: expr) => {
        #[derive(Clone, Copy, Default, Debug)]
        pub struct $name;

        #[async_trait]
        impl TaskResolvent for $name {
            async fn should_count(&self, ctx: &TaskHookContext, result: Option<TaskError>) -> bool {
                $code(ctx, result)
            }
        }
    };
}

implement_core_resolvent!(
    TaskResolveSuccessOnly,
    "0b9473f5-9ce2-49d2-ba68-f4462d605e51",
    (|_ctx: &TaskHookContext, result: Option<TaskError>| result.is_none())
);
implement_core_resolvent!(
    TaskResolveFailureOnly,
    "d5a9db33-9b4e-407e-b2a3-f1487f10be1c",
    (|_ctx: &TaskHookContext, result: Option<TaskError>| result.is_some())
);
implement_core_resolvent!(
    TaskResolveIdentityOnly,
    "053ce742-4ca6-4f32-8bee-6ede0724137d",
    (|_, _| true)
);

/// [`TaskDependencyConfig`] is simply used as a builder to construct [`TaskDependency`], <br />
/// it isn't meant to be used by itself, you may refer to [`TaskDependency::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = TaskDependency))]
pub struct TaskDependencyConfig<T1: TaskFrame, T2: TaskSchedule, T3: ScheduleStrategy> {
    /// The [`Task`] to monitor closely when it finishes a run and act accordingly
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`Task`]
    task: &'static Task<T1, T2, T3>,

    /// The number of relevant runs for the [`TaskDependency`] to be considered resolved
    ///
    /// # Default Value
    /// By default, every [`Task`] requires at least one run for this dependency to be resolved
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskDependency`]
    /// - [`Task`]
    #[builder(default = NonZeroU64::new(1).unwrap())]
    minimum_runs: NonZeroU64,

    /// The task resolvent behavior. This monitors the [`Task`] and whenever it finishes
    /// a run, it checks if this specific run counts towards the relevant counter
    ///
    /// # Default Value
    /// By default, every [`Task`] run will count **ONLY** if it is a successful one
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskResolvent`]
    /// - [`TaskResolveSuccessOnly`]
    #[builder(
        default = Arc::new(TaskResolveSuccessOnly),
        setter(transform = |ts: impl TaskResolvent + 'static| Arc::new(ts) as Arc<dyn TaskResolvent>)
    )]
    resolve_behavior: Arc<dyn TaskResolvent>,
}

struct TaskDependencyTracker {
    run_count: Arc<AtomicU64>,
    minimum_runs: NonZeroU64,
    resolve_behavior: Arc<dyn TaskResolvent>,
}

#[async_trait]
impl TaskHook<OnTaskEnd> for TaskDependencyTracker {
    async fn on_event(
        &self,
        _event: OnTaskEnd,
        ctx: &TaskHookContext,
        payload: &<OnTaskEnd as TaskHookEvent>::Payload,
    ) {
        let should_increment = self
            .resolve_behavior
            .should_count(ctx, payload.clone())
            .await;

        if should_increment {
            return;
        }

        self.run_count.fetch_add(1, Ordering::Relaxed);
    }
}

impl<T1, T2, T3> From<TaskDependencyConfig<T1, T2, T3>> for TaskDependency
where
    T1: TaskFrame,
    T2: TaskSchedule,
    T3: ScheduleStrategy,
{
    fn from(config: TaskDependencyConfig<T1, T2, T3>) -> Self {
        let tracker = Arc::new(TaskDependencyTracker {
            run_count: Arc::new(AtomicU64::default()),
            minimum_runs: config.minimum_runs,
            resolve_behavior: config.resolve_behavior,
        });

        let cloned_tracker = tracker.clone();

        tokio::task::spawn_blocking(move || async move {
            config.task.attach_hook::<OnTaskEnd>(cloned_tracker).await;
        });

        Self {
            task_dependency_tracker: tracker.clone(),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

/// [`TaskDependency`] represents a dependency between a task, typically when a task is executed,
/// the task dependency closely monitors it to see if it succeeds or fails as well as other relevant
/// information. Depending on the configured behavior via [`TaskResolvent`] (count fails, successes or a
/// custom solution), it will count this run towards the resolving of the dependency. If there is
/// a disagreement with the [`TaskResolvent`] and the results, then it doesn't operate at all
///
/// # Constructor(s)
/// When constructing a [`TaskDependency`], one should use [`TaskDependency::builder`] or
/// [`TaskDependency::builder_owned`] depending on if they want to supply either
/// an owned task or a non-owned task
///
/// # Trait Implementation(s)
/// It is obvious that [`TaskDependency`] implements the [`FrameDependency`] trait, but it
/// also implements the extension traits [`ResolvableFrameDependency`] and [`UnresolvableFrameDependency`]
/// for manual resolving / unresolving of the dependency
///
/// # Example
/// ```ignore
/// use std::num::NonZeroU64;
/// use chronographer_core::task::{ExecutionTaskFrame, Task, TaskScheduleImmediate};
/// use chronographer_core::task::dependency::{TaskDependency, TaskResolveIdentityOnly};
///
/// let alpha_task = Task::define(
///     TaskScheduleImmediate,
///     ExecutionTaskFrame::new(|_| {
///         println!("Task Frame A EXECUTED")
///     })
/// );
///
/// let beta_task = Task::define(
///     TaskScheduleImmediate,
///     ExecutionTaskFrame::new(|_| {
///         println!("Task Frame B EXECUTED")
///     })
/// );
///
/// let alpha_dependency = TaskDependency::builder_owned(alpha_task)
///     .minimum_runs(NonZeroU64::new(3).unwrap())
///     .resolve_behavior(TaskResolveIdentityOnly)
///     .build();
/// ```
///
/// # See Also
/// - [`TaskResolvent`]
/// - [`TaskDependency::builder`]
/// - [`TaskDependency::builder_owned`]
/// - [`FrameDependency`]
/// - [`ResolvableFrameDependency`]
/// - [`UnresolvableFrameDependency`]
pub struct TaskDependency {
    task_dependency_tracker: Arc<TaskDependencyTracker>,
    is_enabled: Arc<AtomicBool>,
}

impl TaskDependency {
    /// Creates / Constructs a builder to construct a [`TaskDependency`] instance. There is
    /// a variant of this method for owned task value via [`TaskDependency::builder_owned`]
    ///
    /// # Argument(s)
    /// The method requires one argument, that being a [`Task`] instance wrapped
    /// in an ``Arc<T>``
    ///
    /// # Returns
    /// The builder to construct a [`TaskDependency`]
    ///
    /// # See Also
    /// - [`TaskDependency`]
    /// - [`TaskDependency::builder_owned`]
    pub fn builder<T1, T2, T3>(
        task: &'static Task<T1, T2, T3>,
    ) -> IncompleteTaskDependencyConfig<T1, T2, T3>
    where
        T1: TaskFrame,
        T2: TaskSchedule,
        T3: ScheduleStrategy,
    {
        TaskDependencyConfig::<T1, T2, T3>::builder().task(task)
    }
}

#[async_trait]
impl FrameDependency for TaskDependency {
    async fn is_resolved(&self) -> bool {
        self.task_dependency_tracker
            .run_count
            .load(Ordering::Relaxed)
            >= self.task_dependency_tracker.minimum_runs.get()
    }

    async fn disable(&self) {
        self.is_enabled.store(false, Ordering::Relaxed);
    }

    async fn enable(&self) {
        self.is_enabled.store(true, Ordering::Relaxed);
    }

    async fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl ResolvableFrameDependency for TaskDependency {
    async fn resolve(&self) {
        self.task_dependency_tracker.run_count.store(
            self.task_dependency_tracker.minimum_runs.get(),
            Ordering::Relaxed,
        );
    }
}

#[async_trait]
impl UnresolvableFrameDependency for TaskDependency {
    async fn unresolve(&self) {
        self.task_dependency_tracker
            .run_count
            .store(0, Ordering::Relaxed);
    }
}
