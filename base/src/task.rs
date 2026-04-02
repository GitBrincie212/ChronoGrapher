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
use std::any::TypeId;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock};
use std::sync::atomic::AtomicUsize;

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub struct TaskHookSubmission {
    pub event_type: TypeId,
    pub hook_type: TypeId,
    pub hook: Vec<Arc<dyn ErasedTaskHook>>
}

pub struct Task<T1, T2> {
    frame: T1,
    trigger: T2,
    hooks: Vec<TaskHookSubmission>
}

impl<T1: TaskFrame + Default, T2: TaskTrigger + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            frame: T1::default(),
            trigger: T2::default(),
            hooks: Vec::new()
        }
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn new(frame: T1, trigger: T2) -> Self {
        Self {
            frame,
            trigger,
            hooks: Vec::new()
        }
    }

    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn set_frame(&mut self, new: T1) -> T1 {
        std::mem::replace(&mut self.frame, new)
    }

    pub fn trigger(&self) -> &T2 {
        &self.trigger
    }

    pub fn set_trigger(&mut self, new: T2) -> T2 {
        std::mem::replace(&mut self.trigger, new)
    }

    pub fn attach_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&mut self, hook: Arc<TH>) {
        todo!()
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<&T> {
        todo!()
    }

    pub fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&mut self) {
        todo!()
    }

    pub fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        todo!()
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
}