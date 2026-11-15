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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

fn allocate_instance_id(counter: &AtomicUsize) -> Option<usize> {
    let mut id = counter.load(Ordering::Relaxed);
    loop {
        let next = id.checked_add(1)?;
        match counter.compare_exchange_weak(id, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Some(id),
            Err(actual) => id = actual,
        }
    }
}

pub type ErasedTask<E> = Task<Box<dyn DynTaskFrame<E, ()>>>;

pub struct Task<T1> {
    frame: T1,
    schedule: Box<dyn TaskSchedule>,
    instance_id: usize,
    hook_owner: TaskHookOwner,
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

    pub fn schedule(&self) -> &dyn TaskSchedule {
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
        // Contexts are copyable and may outlive a task. Never reuse an ID.
        let instance_id = allocate_instance_id(&INSTANCE_ID).expect("task instance IDs exhausted");
        Self {
            frame,
            schedule: Box::new(schedule),
            instance_id,
            hook_owner: TaskHookOwner::new(instance_id),
        }
    }

    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn into_erased(self) -> ErasedTask<T1::Error> {
        ErasedTask {
            frame: Box::new(self.frame),
            schedule: self.schedule,
            instance_id: self.instance_id,
            hook_owner: self.hook_owner,
        }
    }
}

pub(crate) trait Sealed {}

#[allow(private_bounds)]
pub trait TaskHookLayer: Sealed + Send + Sync {
    fn attach<EV: TaskHookEvent>(
        &self,
        hook: Arc<impl TaskHook<EV>>,
    ) -> impl Future<Output = ()> + Send;
    fn get<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>>;
    fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>)
    -> impl Future<Output = ()> + Send;
    fn detach<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send;
}

impl<TF: TaskFrame> Sealed for Task<TF> {}

impl<TF: TaskFrame> TaskHookLayer for Task<TF> {
    fn attach<EV: TaskHookEvent>(
        &self,
        hook: Arc<impl TaskHook<EV>>,
    ) -> impl Future<Output = ()> + Send {
        self.attach_hook(hook)
    }

    fn get<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        self.get_hook::<EV, T>()
    }

    fn emit<EV: TaskHookEvent>(
        &self,
        payload: &EV::Payload<'_>,
    ) -> impl Future<Output = ()> + Send {
        self.emit_hook_event::<EV>(payload)
    }

    fn detach<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send {
        self.detach_hook::<EV, T>()
    }
}

#[cfg(test)]
mod tests {
    use super::{AtomicUsize, allocate_instance_id};

    #[test]
    fn exhausted_task_ids_never_wrap() {
        let counter = AtomicUsize::new(usize::MAX - 1);
        assert_eq!(allocate_instance_id(&counter), Some(usize::MAX - 1));
        assert_eq!(allocate_instance_id(&counter), None);
        assert_eq!(allocate_instance_id(&counter), None);
    }
}
