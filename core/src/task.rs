pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod trigger; // skipcq: RS-D1001

pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use trigger::*;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use dashmap::DashMap;
use std::fmt::Debug;
use std::sync::Arc;
use crate::errors::TaskError;

pub type ErasedTask<E> = Task<
    dyn DynTaskFrame<E>,
    dyn TaskTrigger,
>;

pub struct Task<T1: ?Sized + 'static, T2: ?Sized + 'static> {
    frame: Arc<T1>,
    trigger: Arc<T2>,
    hooks: Arc<TaskHookContainer>,
}

impl<T1: TaskFrame + Default, T2: TaskTrigger + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            frame: Arc::new(T1::default()),
            trigger: Arc::new(T2::default()),
            hooks: Arc::new(TaskHookContainer(DashMap::default())),
        }
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn new(schedule: T2, frame: T1) -> Self {
        Self {
            frame: Arc::new(frame),
            trigger: Arc::new(schedule),
            hooks: Arc::new(TaskHookContainer(DashMap::default())),
        }
    }
}

impl<E: TaskError> ErasedTask<E> {
    pub async fn run(&self) -> Result<(), E> {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
        let result: Result<(), E> = self.frame.erased_execute(&ctx).await;
        ctx.emit::<OnTaskEnd>(&result
            .as_ref()
            .err()
            .map(|x| x as &dyn TaskError)
        ).await;

        result
    }

    pub fn frame(&self) -> &dyn DynTaskFrame<E> {
        self.frame.as_ref()
    }

    pub fn trigger(&self) -> &dyn TaskTrigger {
        self.trigger.as_ref()
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn as_erased(&self) -> ErasedTask<T1::Error> {
        ErasedTask {
            frame: self.frame.clone(),
            trigger: self.trigger.clone(),
            hooks: self.hooks.clone(),
        }
    }

    pub async fn attach_hook<E: TaskHookEvent>(&self, hook: Arc<impl TaskHook<E>>) {
        let ctx = TaskHookContext {
            hooks_container: self.hooks.clone(),
            depth: 0,
            frame: self.frame.as_ref(),
        };

        self.hooks.attach(&ctx, hook).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        self.hooks.get::<E, T>()
    }

    pub async fn emit_hook_event<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) {
        let ctx = TaskHookContext {
            hooks_container: self.hooks.clone(),
            depth: 0,
            frame: self.frame.as_ref(),
        };

        self.hooks.emit::<E>(&ctx, payload).await;
    }

    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        let ctx = TaskHookContext {
            hooks_container: self.hooks.clone(),
            depth: 0,
            frame: self.frame.as_ref(),
        };

        self.hooks.detach::<E, T>(&ctx).await;
    }

    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn trigger(&self) -> &T2 {
        &self.trigger
    }
}
