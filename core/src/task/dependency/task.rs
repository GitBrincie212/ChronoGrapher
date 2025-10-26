use crate::persistent_object::PersistentObject;
use crate::retrieve_registers::RetrieveRegistries;
use crate::serialized_component::SerializedComponent;
use crate::task::{Debug, OnTaskEnd, TaskHook, TaskHookEvent};
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{Task, TaskContext, TaskError};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use serde_json::json;
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
    async fn should_count(&self, ctx: Arc<TaskContext>, result: Option<TaskError>) -> bool;
}

macro_rules! implement_core_resolvent {
    ($name: ident, $code: expr) => {
        #[derive(Clone, Copy, Default, Debug)]
        pub struct $name;

        #[async_trait]
        impl TaskResolvent for $name {
            async fn should_count(
                &self,
                ctx: Arc<TaskContext>,
                result: Option<TaskError>,
            ) -> bool {
                $code(ctx, result)
            }
        }

        #[async_trait]
        impl PersistentObject for $name {
            fn persistence_id() -> &'static str {
                concat!(stringify!($name), "$chronographer_core")
            }

            async fn persist(&self) -> Result<SerializedComponent, TaskError> {
                Ok(SerializedComponent::new::<Self>(json!({})))
            }

            async fn retrieve(_component: SerializedComponent) -> Result<Self, TaskError> {
                Ok($name)
            }
        }
    };
}

implement_core_resolvent!(
    TaskResolveSuccessOnly,
    (|_ctx: Arc<TaskContext>, result: Option<TaskError>| result.is_none())
);
implement_core_resolvent!(
    TaskResolveFailureOnly,
    (|_ctx: Arc<TaskContext>, result: Option<TaskError>| result.is_some())
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

struct TaskDependencyTracker {
    run_count: Arc<AtomicU64>,
    minimum_runs: NonZeroU64,
    resolve_behavior: Arc<dyn TaskResolvent>
}

#[async_trait]
impl TaskHook<OnTaskEnd> for TaskDependencyTracker {
    async fn on_event(
        &self, _event: OnTaskEnd,
        ctx: Arc<TaskContext>,
        payload: &<OnTaskEnd as TaskHookEvent>::Payload
    ) {
        let should_increment = self.resolve_behavior
            .should_count(ctx.clone(), payload.clone())
            .await;

        if should_increment {
            return;
        }

        self.run_count.fetch_add(1, Ordering::Relaxed);
    }
}

impl From<TaskDependencyConfig> for TaskDependency {
    fn from(config: TaskDependencyConfig) -> Self {
        let tracker = Arc::new(TaskDependencyTracker {
            run_count: Arc::new(AtomicU64::default()),
            minimum_runs: config.minimum_runs,
            resolve_behavior: config.resolve_behavior,
        });

        let cloned_task = config.task.clone();
        let cloned_tracker = tracker.clone();

        tokio::task::spawn_blocking(move || {
            async move {
                cloned_task.attach_hook::<OnTaskEnd>(cloned_tracker).await;
            }
        });

        Self {
            task: config.task.clone(),
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
    task: Arc<Task>,
    task_dependency_tracker: Arc<TaskDependencyTracker>,
    is_enabled: Arc<AtomicBool>,
}

impl TaskDependency {
    /// Creates / Constructs a builder to construct a [`TaskDependency`] instance. There is
    /// a variant of this method for non-owned task value via [`TaskDependency::builder`]
    ///
    /// # Argument(s)
    /// The method requires one argument, that being an owned [`Task`] instance
    ///
    /// # Returns
    /// The builder to construct a [`TaskDependency`]
    ///
    /// # See Also
    /// - [`TaskDependency`]
    /// - [`TaskDependency::builder`]
    pub fn builder_owned(task: Task) -> TaskDependencyConfigBuilder<((Arc<Task>,), (), ())> {
        TaskDependencyConfig::builder().task(Arc::new(task))
    }

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
    pub fn builder(task: Arc<Task>) -> TaskDependencyConfigBuilder<((Arc<Task>,), (), ())> {
        TaskDependencyConfig::builder().task(task)
    }
}

#[async_trait]
impl FrameDependency for TaskDependency {
    async fn is_resolved(&self) -> bool {
        self.task_dependency_tracker.run_count.load(Ordering::Relaxed)
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
        self.task_dependency_tracker.run_count
            .store(self.task_dependency_tracker.minimum_runs.get(), Ordering::Relaxed);
    }
}

#[async_trait]
impl UnresolvableFrameDependency for TaskDependency {
    async fn unresolve(&self) {
        self.task_dependency_tracker.run_count.store(0, Ordering::Relaxed);
    }
}

#[async_trait]
impl PersistentObject for TaskDependency {
    fn persistence_id() -> &'static str {
        "TaskDependency$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let runs = PersistenceUtils::serialize_field(
            self.task_dependency_tracker.run_count.load(Ordering::Relaxed)
        )?;
        let min_runs = PersistenceUtils::serialize_field(
            self.task_dependency_tracker.minimum_runs.get()
        )?;
        let is_enabled =
            PersistenceUtils::serialize_field(self.is_enabled.load(Ordering::Relaxed))?;
        let task_resolvent =
            PersistenceUtils::serialize_potential_field(
                &self.task_dependency_tracker.resolve_behavior
            ).await?;
        let task_id = PersistenceUtils::serialize_field(self.task.id().as_u128())?;
        Ok(SerializedComponent::new::<Self>(json!({
            "counted_runs": runs,
            "minimum_runs": min_runs,
            "is_enabled": is_enabled,
            "task_resolvent": task_resolvent,
            "task_id": task_id
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;

        let counted_runs = PersistenceUtils::deserialize_atomic::<u64>(
            &mut repr,
            "counted_runs",
            "Cannot deserialize the counted runs",
        )?;

        let min_runs = PersistenceUtils::deserialize_atomic::<u64>(
            &mut repr,
            "minimum_runs",
            "Cannot deserialize the minimum number of runs",
        )?;

        let is_enabled = PersistenceUtils::deserialize_atomic::<bool>(
            &mut repr,
            "is_enabled",
            "Cannot deserialize the data used for indicating if the dependency was enabled or not",
        )?;

        let task_resolvent = PersistenceUtils::deserialize_dyn(
            &mut repr,
            "task_resolvent",
            RetrieveRegistries::retrieve_task_resolvent,
            "Cannot deserialize the task_resolvent",
        )
        .await?;

        Ok(TaskDependency {
            task: Arc::new(Task), // TODO: Find a way to retrieve a task from its ID
            task_dependency_tracker: Arc::new(TaskDependencyTracker {
                minimum_runs: NonZeroU64::new(min_runs).ok_or_else(|| {
                    PersistenceUtils::create_retrieval_error::<u64>(
                        &repr,
                        "Minimum number of runs was set to zero",
                    )
                })?,
                run_count: Arc::new(AtomicU64::new(counted_runs)),
                resolve_behavior: task_resolvent
            }),
            is_enabled: Arc::new(AtomicBool::new(is_enabled)),
        })
    }
}
