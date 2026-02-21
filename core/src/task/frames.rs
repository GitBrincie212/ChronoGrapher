pub mod conditionframe; // skipcq: RS-D1001

pub mod dependencyframe; // skipcq: RS-D1001

pub mod fallbackframe; // skipcq: RS-D1001

pub mod noopframe; // skipcq: RS-D1001

pub mod collectionframe; // skipcq: RS-D1001

pub mod retryframe; // skipcq: RS-D1001

pub mod timeoutframe; // skipcq: RS-D1001

pub mod delayframe; // skipcq: RS-D1001

pub mod dynamicframe; // skipcq: RS-D1001

pub mod thresholdframe; // skipcq: RS-D1001

pub use collectionframe::*;
pub use conditionframe::*;
pub use delayframe::*;
pub use dependencyframe::*;
pub use fallbackframe::*;
pub use noopframe::*;
pub use retryframe::*;
pub use thresholdframe::*;
pub use timeoutframe::*;

use crate::errors::{StandardCoreErrorsCG, TaskError};
use crate::prelude::NonObserverTaskHook;
use crate::task::{ErasedTask, TaskHook, TaskHookContainer, TaskHookContext, TaskHookEvent};
use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use crate::scheduler::engine::default::SchedulerHandleInstructions;
use crate::scheduler::SchedulerHandle;

#[derive(Clone)]
pub struct RestrictTaskFrameContext<'a> {
    pub(crate) hooks_container: Arc<TaskHookContainer>,
    pub(crate) depth: u64,
    pub(crate) frame: &'a dyn ErasedTaskFrame,
}

#[derive(Clone)]
pub struct TaskFrameContext<'a>(pub(crate) RestrictTaskFrameContext<'a>);

macro_rules! instruct_method {
    ($name: ident, $variant: ident) => {
        pub async fn $name(&self) -> Result<(), StandardCoreErrorsCG> {
            if let Some(hook) = self.get_hook::<(), SchedulerHandle>() {
                hook.instruct(SchedulerHandleInstructions::$variant).await;
                return Ok(())
            }
            Err(StandardCoreErrorsCG::SchedulerInstructionsUnsupported)
        }
    };
}

impl<'a> TaskFrameContext<'a> {
    pub(crate) fn subdivided_ctx(&self, frame: &'a dyn ErasedTaskFrame) -> Self {
        Self(RestrictTaskFrameContext {
            hooks_container: self.0.hooks_container.clone(),
            frame,
            depth: self.0.depth + 1,
        })
    }

    pub async fn erased_subdivide(
        &self,
        frame: &'a dyn ErasedTaskFrame,
    ) -> Result<(), Box<dyn TaskError>> {
        let child_ctx = self.subdivided_ctx(frame);
        frame.erased_execute(&child_ctx).await
    }

    pub async fn subdivide<T: TaskFrame>(&self, frame: &'a T) -> Result<(), T::Error> {
        let child_ctx = self.subdivided_ctx(frame);
        frame.execute(&child_ctx).await
    }

    instruct_method!(instruct_reschedule, Reschedule);
    instruct_method!(instruct_block, Block);
    instruct_method!(instruct_halt, Halt);
    instruct_method!(instruct_execute, Execute);

    pub fn as_restricted(&self) -> &RestrictTaskFrameContext<'a> {
        &self.0
    }
}

impl<'a> RestrictTaskFrameContext<'a> {
    pub(crate) fn new(task: &'a ErasedTask<impl TaskError>) -> Self {
        Self {
            hooks_container: task.hooks.clone(),
            depth: 0,
            frame: task.frame.as_ref().erased(),
        }
    }

    pub fn frame(&self) -> &dyn ErasedTaskFrame {
        self.frame
    }

    pub async fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        self.hooks_container
            .emit::<EV>(
                &TaskHookContext::new(self.frame, self.depth, self.hooks_container.clone()),
                payload,
            )
            .await;
    }

    pub async fn attach_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self, hook: Arc<TH>) {
        self.hooks_container
            .attach::<EV, TH>(
                &TaskHookContext::new(self.frame, self.depth, self.hooks_container.clone()),
                hook,
            )
            .await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) {
        self.hooks_container
            .detach::<EV, TH>(&TaskHookContext::new(
                self.frame,
                self.depth,
                self.hooks_container.clone(),
            ))
            .await;
    }

    pub fn get_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) -> Option<Arc<TH>> {
        self.hooks_container.get::<EV, TH>()
    }

    pub async fn shared<H>(&self, creator: impl FnOnce() -> H) -> Arc<H>
    where
        H: NonObserverTaskHook + Send + Sync + 'static,
    {
        if let Some(hook) = self.get_hook::<(), H>() {
            hook
        } else {
            let hook = Arc::new(creator());
            self.attach_hook::<(), H>(hook.clone()).await;
            hook
        }
    }

    pub fn get_shared<H>(&self) -> Option<Arc<H>>
    where
        H: NonObserverTaskHook + Send + Sync + 'static,
    {
        self.get_hook::<(), H>()
    }
}

impl<'a> Deref for TaskFrameContext<'a> {
    type Target = RestrictTaskFrameContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
pub trait TaskFrame: 'static + Send + Sync + Sized {
    type Error: TaskError;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DynTaskFrame<E: TaskError>: 'static + Send + Sync {
    async fn erased_execute(&self, ctx: &TaskFrameContext) -> Result<(), E>;
    fn erased(&self) -> &dyn ErasedTaskFrame;
}

#[async_trait]
impl<T: TaskFrame<Error: Into<T::Error>>> DynTaskFrame<T::Error> for T {
    async fn erased_execute(&self, ctx: &TaskFrameContext) -> Result<(), T::Error> {
        self.execute(ctx).await
    }

    fn erased(&self) -> &dyn ErasedTaskFrame {
        self
    }
}

#[async_trait]
pub trait ErasedTaskFrame: 'static + Send + Sync {
    async fn erased_execute(&self, ctx: &TaskFrameContext) -> Result<(), Box<dyn TaskError>>;
}

#[async_trait]
impl<T: TaskFrame<Error: Into<T::Error>>> ErasedTaskFrame for T {
    async fn erased_execute(&self, ctx: &TaskFrameContext) -> Result<(), Box<dyn TaskError>> {
        self.execute(ctx)
            .await
            .map_err(|x| Box::new(x) as Box<dyn TaskError>)
    }
}
