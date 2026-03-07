pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod trigger; // skipcq: RS-D1001

pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use trigger::*;

use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, LazyLock};

static HOOK_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub type ErasedTask<E> = Task<dyn DynTaskFrame<E>, dyn TaskTrigger>;

pub struct Task<T1: ?Sized + 'static, T2: ?Sized + 'static> {
    frame: Arc<T1>,
    trigger: Arc<T2>,
    hook_id: usize,
}

impl<T1: TaskFrame + Default, T2: TaskTrigger + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            frame: Arc::new(T1::default()),
            trigger: Arc::new(T2::default()),
            hook_id: HOOK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

impl<E: TaskError> ErasedTask<E> {
    pub async fn run(&self) -> Result<(), E> {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
        let result: Result<(), E> = self.frame.erased_execute(&ctx).await;
        ctx.emit::<OnTaskEnd>(&result.as_ref().map_err(|x| x as &dyn TaskError).err())
            .await;

        result
    }

    pub fn frame(&self) -> &Arc<dyn DynTaskFrame<E>> {
        &self.frame
    }

    pub fn trigger(&self) -> &Arc<dyn TaskTrigger> {
        &self.trigger
    }

    pub fn set_frame(&mut self, new_frame: impl TaskFrame<Error = E>) {
        self.frame = Arc::new(new_frame) as Arc<dyn DynTaskFrame<E>>
    }

    pub fn set_trigger(&mut self, new_trigger: impl TaskTrigger) {
        self.trigger = Arc::new(new_trigger) as Arc<dyn TaskTrigger>
    }

    pub async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        let ctx = TaskHookContext {
            depth: 0,
            task_id: self.hook_id,
            frame: self.frame.erased(),
        };

        ctx.attach_hook(hook).await;
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.hook_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext {
            task_id: self.hook_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        ctx.emit::<EV>(payload).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext {
            task_id: self.hook_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        ctx.detach_hook::<EV, T>().await;
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn new(schedule: T2, frame: T1) -> Self {
        Self {
            frame: Arc::new(frame),
            trigger: Arc::new(schedule),
            hook_id: HOOK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub fn as_erased(&self) -> ErasedTask<T1::Error> {
        ErasedTask {
            frame: self.frame.clone(),
            trigger: self.trigger.clone(),
            hook_id: self.hook_id,
        }
    }

    pub fn frame(&self) -> &Arc<T1> {
        &self.frame
    }

    pub fn trigger(&self) -> &Arc<T2> {
        &self.trigger
    }

    pub async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        let ctx = TaskHookContext {
            task_id: self.hook_id,
            depth: 0,
            frame: self.frame.as_ref(),
        };

        ctx.attach_hook(hook).await;
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.hook_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext {
            task_id: self.hook_id,
            depth: 0,
            frame: self.frame.as_ref(),
        };

        ctx.emit::<EV>(payload).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext {
            task_id: self.hook_id,
            depth: 0,
            frame: self.frame.as_ref(),
        };

        ctx.detach_hook::<EV, T>().await;
    }
}
