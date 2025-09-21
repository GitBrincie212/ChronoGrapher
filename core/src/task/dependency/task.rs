use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{Task, TaskError};
use async_trait::async_trait;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use typed_builder::TypedBuilder;

/// [`TaskResolvent`] acts as a policy dictating how the [`TaskDependency`] manages the counting
/// of runs, there are 3 core implementations of this trait which are specifically:
/// - [`TaskResolveSuccessOnly`] counts only successful runs
/// - [`TaskResolveFailuresOnly`] counts only failure runs
/// - [`TaskResolveIdentity`] counts both successful and failure runs,
///
/// by default [`TaskDependency`] uses [`TaskResolveSuccessOnly`] (which of course can be overridden)
#[async_trait]
pub trait TaskResolvent: Send + Sync {
    async fn should_count(&self, result: Arc<Option<TaskError>>) -> bool;
}

macro_rules! implement_core_resolvent {
    ($name: ident, $code: expr) => {
        pub struct $name;

        #[async_trait]
        impl TaskResolvent for $name {
            async fn should_count(&self, result: Arc<Option<TaskError>>) -> bool {
                $code(result)
            }
        }
    };
}

implement_core_resolvent!(
    TaskResolveSuccessOnly,
    (|result: Arc<Option<TaskError>>| result.is_none())
);
implement_core_resolvent!(
    TaskResolveFailureOnly,
    (|result: Arc<Option<TaskError>>| result.is_some())
);
implement_core_resolvent!(TaskResolveIdentityOnly, (|_| true));

#[derive(TypedBuilder)]
#[builder(build_method(into = TaskDependency))]
pub struct TaskDependencyConfig {
    task: Arc<Task>,

    #[builder(default = NonZeroU64::new(1).unwrap())]
    minimum_runs: NonZeroU64,

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
                    .frame
                    .on_end()
                    .subscribe(move |_, payload: Arc<Option<TaskError>>| {
                        let counter_clone = counter_clone.clone();
                        let resolve_behavior_clone = resolve_behavior_clone.clone();

                        async move {
                            let should_increment =
                                resolve_behavior_clone.should_count(payload.clone()).await;
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
/// the task dependency closely monitors it to see if it succeeds or fails. Depending on the configured
/// behavior (count fails, successes or a custom solution), it will count this run towards the resolving
/// of the dependency. If there is a disagreement with the configured behavior and the results, then
/// it doesn't operate at all
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
