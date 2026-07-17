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

pub type ErasedTask<E> = Task<Box<dyn DynTaskFrame<E, ()>>>;

pub struct Task<T1> {
    frame: T1,
    schedule: Box<dyn TaskSchedule>,
    instance_id: usize
}

impl<T1> Task<T1> {
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

    pub fn schedule(&self) -> &dyn TaskSchedule  {
        self.schedule.as_ref()
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
}

impl<T1: TaskFrame<Args = ()>> Task<T1> {
    pub fn new(frame: T1, schedule: impl TaskSchedule) -> Self {
        Self {
            frame,
            schedule: Box::new(schedule),
            instance_id: INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }
    }

    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn into_erased(self) -> ErasedTask<T1::Error> {
        ErasedTask {
            frame: Box::new(self.frame),
            schedule: self.schedule,
            instance_id: self.instance_id
        }
    }
}

pub(crate) trait Sealed {}

#[allow(private_interfaces)]
pub trait TaskHookLayer: Sealed + Send + Sync {
    fn attach<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) -> impl Future<Output=()> + Send;
    fn get<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>>;
    fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output=()> + Send;
    fn detach<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output=()> + Send;
}

impl<TF: TaskFrame> Sealed for Task<TF> {}

impl<TF: TaskFrame> TaskHookLayer for Task<TF> {
    fn attach<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) -> impl Future<Output=()> + Send {
        self.attach_hook(hook)
    }

    fn get<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        self.get_hook::<EV, T>()
    }

    fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) -> impl Future<Output=()> + Send {
        self.emit_hook_event::<EV>(payload)
    }

    fn detach<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output=()> + Send {
        self.detach_hook::<EV, T>()
    }
}