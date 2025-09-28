use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{Task, TaskContext, TaskError};
use async_trait::async_trait;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use typed_builder::TypedBuilder;

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
    async fn should_count(&self,  ctx: Arc<TaskContext<true>>, result: Option<TaskError>) -> bool;
}

macro_rules! implement_core_resolvent {
    ($name: ident, $code: expr) => {
        pub struct $name;

        #[async_trait]
        impl TaskResolvent for $name {
            async fn should_count(&self, ctx: Arc<TaskContext<true>>, result: Option<TaskError>) -> bool {
                $code(ctx, result)
            }
        }
    };
}

implement_core_resolvent!(
    TaskResolveSuccessOnly,
    (|_ctx: Arc<TaskContext<true>>, result: Option<TaskError>| result.is_none())
);
implement_core_resolvent!(
    TaskResolveFailureOnly,
    (|_ctx: Arc<TaskContext<true>>, result: Option<TaskError>| result.is_some())
);
implement_core_resolvent!(TaskResolveIdentityOnly, (|_, _| true));

/// [`TaskDependencyConfig`] is simply used as a builder to construct [`TaskDependency`], <br />
/// it isn't meant to be used by itself, you may refer to [`TaskDependency::builder`]
#[derive(TypedBuilder)]
#[builder(build_method(into = TaskDependency))]
pub struct TaskDependencyConfig {
    /// The [`Task`] to monitor closely when it finishes a run and act accordingly
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`Task`]
    task: Arc<Task>,

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

impl From<TaskDependencyConfig> for TaskDependency {
    fn from(config: TaskDependencyConfig) -> Self {
        let slf = Self {
            task: config.task,
            minimum_runs: config.minimum_runs,
            counter: Arc::new(AtomicU64::new(0)),
            is_enabled: Arc::new(AtomicBool::new(true)),
            resolve_behavior: config.resolve_behavior,
        };

        let counter_clone = slf.counter.clone();
        let resolve_behavior_clone = slf.resolve_behavior.clone();
        let task_clone = slf.task.clone();

        tokio::task::spawn_blocking(move || {
            let counter_clone = counter_clone.clone();
            let resolve_behavior_clone = resolve_behavior_clone.clone();
            let task_clone = task_clone.clone();

            async move {
                task_clone
                    .on_end
                    .subscribe(move |ctx: Arc<TaskContext<true>>, payload: Arc<Option<TaskError>>| {
                        let counter_clone = counter_clone.clone();
                        let resolve_behavior_clone = resolve_behavior_clone.clone();
                        let payload_cloned = payload.as_ref().clone();
                        let context_cloned = ctx.clone();

                        async move {
                            let should_increment =
                                resolve_behavior_clone.should_count(context_cloned, payload_cloned).await;
                            if should_increment {
                                return;
                            }
                            counter_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                    .await;
            }
        });

        slf
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
    task: Arc<Task>,
    minimum_runs: NonZeroU64,
    counter: Arc<AtomicU64>,
    is_enabled: Arc<AtomicBool>,
    resolve_behavior: Arc<dyn TaskResolvent>,
}

impl TaskDependency {
    pub fn builder_owned(task: Task) -> TaskDependencyConfigBuilder<((Arc<Task>,), (), ())> {
        TaskDependencyConfig::builder().task(Arc::new(task))
    }

    pub fn builder(task: Arc<Task>) -> TaskDependencyConfigBuilder<((Arc<Task>,), (), ())> {
        TaskDependencyConfig::builder().task(task)
    }
}

#[async_trait]
impl FrameDependency for TaskDependency {
    async fn is_resolved(&self) -> bool {
        self.counter.load(Ordering::Relaxed) >= self.minimum_runs.get()
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
        self.counter
            .store(self.minimum_runs.get(), Ordering::Relaxed);
    }
}

#[async_trait]
impl UnresolvableFrameDependency for TaskDependency {
    async fn unresolve(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}
