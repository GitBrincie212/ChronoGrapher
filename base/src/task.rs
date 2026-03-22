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
use std::ops::Deref;
use std::sync::{Arc, LazyLock, Weak};
use std::sync::atomic::AtomicUsize;
use async_trait::async_trait;
use crate::scheduler::{assign_dispatching_to_worker, assign_triggering_to_worker, SchedulerConfig, SchedulerWorker};
use crate::scheduler::task_store::SchedulerTaskStore;

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

#[async_trait]
pub trait TaskRef<C: SchedulerConfig>: Clone + Send + Sync {
    type TaskFrame<'a>: DynTaskFrame<C::TaskError> where Self: 'a;
    type TaskTrigger<'a>: TaskTrigger where Self: 'a;

    async fn frame(&self) -> Option<Self::TaskFrame<'_>>;
    async fn trigger(&self) -> Option<Self::TaskTrigger<'_>>;

    async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>);
    async fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>>;
    async fn emit_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>);
    async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self);

    fn is_invalid(&self) -> bool;
    async fn cancelled(&self);
    async fn cancel(&self);
}

pub struct TaskHandle<C: SchedulerConfig> {
    store: Weak<C::SchedulerTaskStore>,
    dispatcher: Weak<C::SchedulerTaskDispatcher>,
    engine: Weak<C::SchedulerEngine>,
    workers: Weak<Vec<SchedulerWorker<C>>>,
    handle: <C::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskRef,
}

impl<C: SchedulerConfig> Clone for TaskHandle<C> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            dispatcher: self.dispatcher.clone(),
            engine: self.engine.clone(),
            workers: self.workers.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<C: SchedulerConfig> TaskHandle<C> {
    pub(crate) fn new(
        store: Weak<C::SchedulerTaskStore>,
        dispatcher: Weak<C::SchedulerTaskDispatcher>,
        engine: Weak<C::SchedulerEngine>,
        workers: Weak<Vec<SchedulerWorker<C>>>,
        handle: <C::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskRef,
    ) -> Self {
        Self {
            store,
            dispatcher,
            engine,
            workers,
            handle
        }
    }

    #[inline(always)]
    pub fn schedule(&self) {
        self.clone().schedule_owned()
    }

    #[inline(always)]
    pub fn schedule_owned(self) {
        if let Some(workers) = self.workers.upgrade() {
            assign_triggering_to_worker::<C>(
                self,
                workers.as_ref()
            );
        }
    }

    #[inline(always)]
    pub async fn remove(&self) {
        if let Some(store) = self.store.upgrade() {
            store.deallocate(&self.handle).await;
        }
    }

    #[inline(always)]
    pub fn dispatch(&self) {
        self.clone().dispatch_owned()
    }

    #[inline(always)]
    pub fn dispatch_owned(self) {
        if let Some(workers) = self.workers.upgrade() {
            assign_dispatching_to_worker(self, workers.as_ref());
        }
    }

    #[inline(always)]
    pub async fn run(&self) -> Result<(), C::TaskError> {
        if let Some(frame) = self.handle.frame().await  {
            let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
            ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
            let result: Result<(), C::TaskError> = frame.erased_execute(&ctx).await;
            ctx.emit::<OnTaskEnd>(
                &result
                    .as_ref()
                    .map_err(|x| x as &dyn TaskError).err()
            ).await;

            return result;
        }

        Ok(())
    }
}

impl<C: SchedulerConfig> Deref for TaskHandle<C> {
    type Target = <C::SchedulerTaskStore as SchedulerTaskStore<C>>::TaskRef;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

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
    pub async fn run(&self) -> Result<(), E> {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
        let result: Result<(), E> = self.frame.erased_execute(&ctx).await;
        ctx.emit::<OnTaskEnd>(&result.as_ref().map_err(|x| x as &dyn TaskError).err())
            .await;

        result
    }

    pub fn frame(&self) -> &dyn DynTaskFrame<E> {
        self.frame.as_ref()
    }

    pub fn trigger(&self) -> &dyn TaskTrigger  {
        self.trigger.as_ref()
    }

    pub async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        let ctx = TaskHookContext {
            depth: 0,
            instance_id: self.instance_id,
            frame: self.frame.erased(),
        };

        ctx.attach_hook(hook).await;
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.instance_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        ctx.emit::<EV>(payload).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: self.frame.erased(),
        };

        ctx.detach_hook::<EV, T>().await;
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

    pub async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        ctx.attach_hook(hook).await;
    }

    pub fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<EV, T>(self.instance_id)
    }

    pub async fn emit_hook_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        ctx.emit::<EV>(payload).await;
    }

    pub async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        let ctx = TaskHookContext {
            instance_id: self.instance_id,
            depth: 0,
            frame: &self.frame,
        };

        ctx.detach_hook::<EV, T>().await;
    }
}
