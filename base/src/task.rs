pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod schedule; // skipcq: RS-D1001

pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use schedule::*;

use crate::errors::TaskError;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock};
use std::sync::atomic::AtomicUsize;

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub type ErasedTask<E> = Task<Box<dyn DynTaskFrame<E, ()>>, Box<dyn TaskSchedule>>;

pub struct Task<T1, T2> {
    frame: T1,
    trigger: T2,
    instance_id: usize
}

impl<T1: TaskFrame + Default, T2: TaskSchedule + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            frame: T1::default(),
            trigger: T2::default(),
            instance_id: INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }
    }
}

impl<T1, T2> Task<T1, T2> {
    pub async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        let ctx = TaskHookContext(self.instance_id);

        ctx.attach_hook(hook).await;
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.instance_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext(self.instance_id);

        ctx.emit::<EV>(payload).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext(self.instance_id);

        ctx.detach_hook::<EV, T>().await;
    }
}

impl<E: TaskError> ErasedTask<E> {
    pub async fn run(&self) -> Result<(), E> {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015

        let result = self.frame.erased_execute(&ctx, &()).await;
        let err = match &result {
            Ok(_) => None,
            Err(e) => Some(e as &dyn TaskError),
        };

        ctx.emit::<OnTaskEnd>(&err).await;
        result
    }

    pub fn frame(&self) -> &dyn DynTaskFrame<E, ()> {
        self.frame.as_ref()
    }

    pub fn schedule(&self) -> &dyn TaskSchedule  {
        self.trigger.as_ref()
    }
}

impl<T1: TaskFrame<Args = ()>, T2: TaskSchedule> Task<T1, T2> {
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
}