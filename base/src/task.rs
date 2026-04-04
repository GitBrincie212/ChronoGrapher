pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod trigger; // skipcq: RS-D1001

pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use trigger::*;
pub use schedule::*;

use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock};
use std::sync::atomic::AtomicUsize;

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub type ErasedTask<E> = Task<Box<dyn DynTaskFrame<E>>, Box<dyn TaskTrigger>>;

pub struct Task<T1, T2> {
    frame: T1,
    trigger: T2,
    instance_id: usize
}

impl<T1: TaskFrame + Default, T2: TaskTrigger + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            frame: T1::default(),
            trigger: T2::default(),
            instance_id: INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }
    }
}

impl<E: TaskError> ErasedTask<E> {
    pub fn run(&self) -> impl Future<Output = Result<(), E>> + Send {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        async move {
            ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
            let result: Result<(), E> = self.frame.erased_execute(&ctx).await;
            let err = result.as_ref().err().map(|x| x as &dyn TaskError);
            ctx.emit::<OnTaskEnd>(&err).await;

            result
        }
    }

    pub fn frame(&self) -> &dyn DynTaskFrame<E> {
        self.frame.as_ref()
    }

    pub fn trigger(&self) -> &dyn TaskTrigger  {
        self.trigger.as_ref()
    }

    pub fn attach_hook<EV: TaskHookEvent>(
        &self, hook: Arc<impl TaskHook<EV>>
    ) -> impl Future<Output = ()> + Send {
        let ctx = TaskHookContext {
            depth: 0,
            instance_id: self.instance_id,
            frame: self.frame.erased(),
        };

        async move { ctx.attach_hook(hook).await; }
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.instance_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        async move { ctx.emit::<EV>(payload).await; }
    }

    pub fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        async move { ctx.detach_hook::<EV, T>().await; }
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn new(trigger: T2, frame: T1) -> Self {
        Self {
            frame,
            trigger,
            instance_id: INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }
    }

    pub fn into_erased(self) -> ErasedTask<T1::Error> {
        ErasedTask {
            frame: Box::new(self.frame),
            trigger: Box::new(self.trigger),
            instance_id: self.instance_id
        }
    }

    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn trigger(&self) -> &T2 {
        &self.trigger
    }

    pub fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) -> impl Future<Output = ()> + Send {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        async move { ctx.attach_hook(hook).await; }
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.instance_id)
    }

    pub fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output = ()> + Send {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        async move { ctx.emit::<EV>(payload).await; }
    }

    pub fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send  {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        async move { ctx.detach_hook::<EV, T>().await; }
    }
}