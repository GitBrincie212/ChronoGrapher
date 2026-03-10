use crate::errors::TaskError;
use crate::task::Task;
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::task::{
    Debug, OnTaskEnd, TaskFrame, TaskHook, TaskHookContext, TaskHookEvent, TaskTrigger,
};
use async_trait::async_trait;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
            async fn should_count(
                &self,
                ctx: &TaskHookContext,
                result: &Option<&dyn TaskError>,
            ) -> bool {
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

pub struct TaskDependencyBuilder<T1: TaskFrame, T2: TaskTrigger> {
    task: &'static Task<T1, T2>,
    minimum_runs: NonZeroU64,
    resolve_behavior: Arc<dyn TaskResolvent>,
}

impl<T1: TaskFrame, T2: TaskTrigger> TaskDependencyBuilder<T1, T2> {
    fn new(task: &'static Task<T1, T2>) -> Self {
        Self {
            task,
            minimum_runs: NonZeroU64::new(1).unwrap(),
            resolve_behavior: Arc::new(TaskResolveSuccessOnly),
        }
    }

    pub fn minimum_runs(mut self, value: NonZeroU64) -> Self {
        self.minimum_runs = value;
        self
    }

    pub fn resolve_behavior(mut self, value: impl TaskResolvent + 'static) -> Self {
        self.resolve_behavior = Arc::new(value);
        self
    }
    
    pub async fn build(self) -> TaskDependency {
        let tracker = Arc::new(TaskDependencyTracker {
            run_count: Arc::new(AtomicU64::default()),
            minimum_runs: self.minimum_runs,
            resolve_behavior: self.resolve_behavior,
        });

        let cloned_tracker = tracker.clone();

        self.task.attach_hook::<OnTaskEnd>(cloned_tracker).await;

        TaskDependency {
            task_dependency_tracker: tracker.clone(),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }
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
        let should_increment = self.resolve_behavior.should_count(ctx, payload).await;

        if !should_increment {
            return;
        }

        self.run_count.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct TaskDependency {
    task_dependency_tracker: Arc<TaskDependencyTracker>,
    is_enabled: Arc<AtomicBool>,
}

impl TaskDependency {
    pub async fn new<T1: TaskFrame, T2: TaskTrigger>(
        task: &'static Task<T1, T2>,
    ) -> Self {
        TaskDependencyBuilder::<T1, T2>::new(task).build().await
    }
    
    pub fn builder<T1: TaskFrame, T2: TaskTrigger>(
        task: &'static Task<T1, T2>,
    ) -> TaskDependencyBuilder<T1, T2> {
        TaskDependencyBuilder::<T1, T2>::new(task)
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
