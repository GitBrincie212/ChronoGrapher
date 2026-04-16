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

use crate::errors::TaskError;
use crate::task::{ErasedTask, NonObserverTaskHook, TaskHook, TaskHookContext, TaskHookEvent, TASKHOOK_REGISTRY};
use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use crate::scheduler::utils::{SchedulerHandleInstructions, SchedulerHandle};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RestrictTaskFrameContext(usize);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaskFrameContext(pub(crate) RestrictTaskFrameContext);

macro_rules! instruct_method {
    ($name: ident, $variant: ident) => {
        pub fn $name(&self) {
            let hook = self.get_hook::<(), SchedulerHandle>().expect("The SchedulerHandle isn't present when its supposed to be");
            hook.instruct(SchedulerHandleInstructions::$variant);
        }
    };
}

impl TaskFrameContext {
    instruct_method!(instruct_reschedule, Reschedule);
    instruct_method!(instruct_block, Block);
    instruct_method!(instruct_halt, Halt);
    instruct_method!(instruct_execute, Execute);

    pub fn as_restricted(&self) -> &RestrictTaskFrameContext {
        &self.0
    }
}

impl RestrictTaskFrameContext {
    pub(crate) fn new(task: &ErasedTask<impl TaskError>) -> Self {
        Self(task.instance_id)
    }

    pub async fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext(self.0);

        ctx.emit::<EV>(payload).await;
    }

    pub async fn attach_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self, hook: Arc<TH>) {
        let ctx = TaskHookContext(self.0);

        ctx.attach_hook::<EV, TH>(hook).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext(self.0);

        ctx.detach_hook::<EV, TH>().await;
    }

    pub fn get_hook<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) -> Option<Arc<TH>> {
        TASKHOOK_REGISTRY.get::<EV, TH>(self.0)
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

impl Deref for TaskFrameContext {
    type Target = RestrictTaskFrameContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait TaskFrame: 'static + Send + Sync + Sized {
    type Error: TaskError;
    type Args: Send + Sync + 'static;

    fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[async_trait]
pub trait DynTaskFrame<E: TaskError, Args: Send + Sync + 'static>: 'static + Send + Sync {
    async fn erased_execute(&self, ctx: &TaskFrameContext, args: &Args) -> Result<(), E>;
    fn erased(&self) -> &dyn ErasedTaskFrame<Args>;
}

#[async_trait]
impl<T: TaskFrame<Error: Into<T::Error>>> DynTaskFrame<T::Error, T::Args> for T {
    async fn erased_execute(&self, ctx: &TaskFrameContext, args: &T::Args) -> Result<(), T::Error> {
        self.execute(ctx, args).await
    }

    fn erased(&self) -> &dyn ErasedTaskFrame<T::Args> {
        self
    }
}

#[async_trait]
pub trait ErasedTaskFrame<Args: Send + Sync + 'static>: 'static + Send + Sync {
    async fn erased_execute(&self, ctx: &TaskFrameContext, args: &Args) -> Result<(), Box<dyn TaskError>>;
}

#[async_trait]
impl<T: TaskFrame<Error: Into<T::Error>>> ErasedTaskFrame<T::Args> for T {
    async fn erased_execute(&self, ctx: &TaskFrameContext, args: &T::Args) -> Result<(), Box<dyn TaskError>> {
        self.execute(ctx, args)
            .await
            .map_err(|x| Box::new(x) as Box<dyn TaskError>)
    }
}