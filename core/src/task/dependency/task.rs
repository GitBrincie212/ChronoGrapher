use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{
    Debug, OnTaskEnd, TaskFrame, TaskHook, TaskHookContext, TaskHookEvent, TaskTrigger,
};
use crate::task::Task;
use async_trait::async_trait;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use typed_builder::TypedBuilder;
use crate::errors::TaskError;

type IncompleteTaskDependencyConfig<T1, T2> =
    TaskDependencyConfigBuilder<T1, T2, ((&'static Task<T1, T2>,), (), ())>;

#[async_trait]
pub trait TaskResolvent: Send + Sync {
    async fn should_count(&self, ctx: &TaskHookContext, result: &Option<&dyn TaskError>) -> bool;
}

macro_rules! implement_core_resolvent {
    ($name: ident, $uuid: expr, $code: expr) => {
        #[derive(Clone, Copy, Default, Debug)]
        pub struct $name;

        #[async_trait]
        impl TaskResolvent for $name {
            async fn should_count(&self, ctx: &TaskHookContext, result: &Option<&dyn TaskError>) -> bool {
                $code(ctx, result)
            }
        }
    };
}

implement_core_resolvent!(
    TaskResolveSuccessOnly,
    "0b9473f5-9ce2-49d2-ba68-f4462d605e51",
    (|_ctx: &TaskHookContext, result: &Option<&dyn TaskError>| result.is_none())
);
implement_core_resolvent!(
    TaskResolveFailureOnly,
    "d5a9db33-9b4e-407e-b2a3-f1487f10be1c",
    (|_ctx: &TaskHookContext, result: &Option<&dyn TaskError>| result.is_some())
);
implement_core_resolvent!(
    TaskResolveIdentityOnly,
    "053ce742-4ca6-4f32-8bee-6ede0724137d",
    (|_, _| true)
);

#[derive(TypedBuilder)]
#[builder(build_method(into = TaskDependency))]
pub struct TaskDependencyConfig<T1: TaskFrame, T2: TaskTrigger> {
    task: &'static Task<T1, T2>,

    #[builder(default = NonZeroU64::new(1).unwrap())]
    minimum_runs: NonZeroU64,

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
        ctx: &TaskHookContext,
        payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>,
    ) {
        let should_increment = self
            .resolve_behavior
            .should_count(ctx, payload)
            .await;

        if should_increment {
            return;
        }

        self.run_count.fetch_add(1, Ordering::Relaxed);
    }
}

impl<T1, T2> From<TaskDependencyConfig<T1, T2>> for TaskDependency
where
    T1: TaskFrame,
    T2: TaskTrigger,
{
    fn from(config: TaskDependencyConfig<T1, T2>) -> Self {
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

pub struct TaskDependency {
    task_dependency_tracker: Arc<TaskDependencyTracker>,
    is_enabled: Arc<AtomicBool>,
}

impl TaskDependency {
    pub fn builder<T1: TaskFrame, T2: TaskTrigger>(
        task: &'static Task<T1, T2>,
    ) -> IncompleteTaskDependencyConfig<T1, T2> {
        TaskDependencyConfig::<T1, T2>::builder().task(task)
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
