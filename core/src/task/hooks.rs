use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

pub mod events {
    pub use crate::task::OnTaskEnd;
    pub use crate::task::OnTaskStart;
    pub use crate::task::frames::ChildTaskFrameEvents;
    pub use crate::task::frames::ConditionalPredicateEvents;
    pub use crate::task::frames::DelayEvents;
    pub use crate::task::frames::OnChildTaskFrameEnd;
    pub use crate::task::frames::OnChildTaskFrameStart;
    pub use crate::task::frames::OnDelayEnd;
    pub use crate::task::frames::OnDelayStart;
    pub use crate::task::frames::OnDependencyValidation;
    pub use crate::task::frames::OnFallbackEvent;
    pub use crate::task::frames::OnFalseyValueEvent;
    pub use crate::task::frames::OnRetryAttemptEnd;
    pub use crate::task::frames::OnRetryAttemptStart;
    pub use crate::task::frames::OnTimeout;
    pub use crate::task::frames::OnTruthyValueEvent;
    pub use crate::task::frames::RetryAttemptEvents;
    pub use crate::task::hooks::OnHookAttach;
    pub use crate::task::hooks::OnHookDetach;
    pub use crate::task::hooks::TaskHookEvent;
    pub use crate::task::hooks::TaskHookLifecycleEvents;
    pub use crate::task::hooks::TaskLifecycleEvents;
} // skipcq: RS-D1001

pub trait TaskHookEvent: Send + Sync + Default + 'static {
    type Payload<'a>: Send + Sync
    where
        Self: 'a;
    const EVENT_ID: &'static str;
}

pub enum NonEmittable {}

impl TaskHookEvent for () {
    type Payload<'a>
        = NonEmittable
    where
        Self: 'a;
    const EVENT_ID: &'static str = "";
}

#[async_trait]
pub trait TaskHook<E: TaskHookEvent>: Send + Sync + 'static {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &E::Payload<'_>) {}
}

pub trait NonObserverTaskHook: Send + Sync + 'static {}

#[async_trait]
impl<T: NonObserverTaskHook> TaskHook<()> for T {}

#[derive(Clone)]
struct ErasedTaskHookWrapper<E: TaskHookEvent> {
    hook: Arc<dyn TaskHook<E>>,
    concrete: Arc<dyn Any + Send + Sync>,
    _marker: PhantomData<E>,
}

impl<E: TaskHookEvent> ErasedTaskHookWrapper<E> {
    pub fn new<T: TaskHook<E>>(hook: Arc<T>) -> Self {
        Self {
            hook: hook.clone(),
            concrete: hook,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
pub(crate) trait ErasedTaskHook: Send + Sync {
    async fn on_emit<'a>(&self, ctx: &TaskHookContext, payload: ErasedPayload<'a>);
    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

#[async_trait]
impl<E: TaskHookEvent + 'static> ErasedTaskHook for ErasedTaskHookWrapper<E> {
    async fn on_emit<'a>(&self, ctx: &TaskHookContext, payload: ErasedPayload<'a>) {
        let payload = unsafe { payload.cast() };

        self.hook.on_event(ctx, payload).await;
    }

    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        // Return the original concrete hook, not the wrapper
        self.concrete.clone()
    }
}

define_event!(OnTaskStart, ());

define_event!(OnTaskEnd, Option<&'a dyn TaskError>);

define_event_group!(TaskLifecycleEvents, OnTaskStart, OnTaskEnd);

macro_rules! define_hook_event {
    ($(#[$($attrs:tt)*])* $name: ident) => {
        $(#[$($attrs)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name<E: TaskHookEvent>(PhantomData<E>);

        impl<E: TaskHookEvent> Default for $name<E> {
            fn default() -> Self {
                $name(PhantomData)
            }
        }

        impl<E: TaskHookEvent> TaskHookEvent for $name<E> {
            type Payload<'a> = Arc<dyn TaskHook<E>> where Self: 'a;
            const EVENT_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

define_hook_event!(OnHookAttach);

define_hook_event!(OnHookDetach);

pub trait TaskHookLifecycleEvents<E: TaskHookEvent>:
    TaskHookEvent<Payload<'static> = Arc<dyn TaskHook<E>>>
{
}

impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookAttach<E> {}
impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookDetach<E> {}

#[derive(Default)]
pub(crate) struct TaskHookEventCategory(pub DashMap<TypeId, Arc<dyn ErasedTaskHook>>);

#[derive(Clone)]
pub struct TaskHookContext<'a> {
    pub(crate) depth: u64,
    pub(crate) hooks_container: Arc<TaskHookContainer>,
    pub(crate) frame: &'a dyn ErasedTaskFrame,
}

impl<'a> TaskHookContext<'a> {
    pub(crate) fn new(
        frame: &'a dyn ErasedTaskFrame,
        depth: u64,
        hooks_container: Arc<TaskHookContainer>,
    ) -> Self {
        Self {
            depth,
            hooks_container,
            frame,
        }
    }

    pub fn depth(&self) -> u64 {
        self.depth
    }

    pub fn frame(&self) -> &dyn ErasedTaskFrame {
        self.frame
    }

    pub async fn emit<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) {
        self.hooks_container.emit::<E>(self, payload).await;
    }

    pub async fn attach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self, hook: Arc<T>) {
        self.hooks_container.attach::<E, T>(self, hook).await;
    }

    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        self.hooks_container.detach::<E, T>(self).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        self.hooks_container.get::<E, T>()
    }
}

pub struct ErasedPayload<'a>(&'a (dyn Send + Sync));

impl<'a> ErasedPayload<'a> {
    pub fn new<T: Sized + Send + Sync + 'a>(payload: &'a T) -> Self {
        Self(payload)
    }

    unsafe fn cast<T: Sized + Send + Sync + 'a>(&self) -> &'a T {
        unsafe { &*(self.0 as *const (dyn Send + Sync) as *const T) }
    }
}

pub struct TaskHookContainer(pub(crate) DashMap<&'static str, TaskHookEventCategory>);

impl TaskHookContainer {
    pub async fn attach<E: TaskHookEvent, T: TaskHook<E>>(
        &self,
        ctx: &TaskHookContext<'_>,
        hook: Arc<T>,
    ) {
        let hook_id = TypeId::of::<T>();
        let erased_hook: Arc<dyn ErasedTaskHook> =
            Arc::new(ErasedTaskHookWrapper::<E>::new(hook.clone()));

        self.0
            .entry(E::EVENT_ID)
            .or_default()
            .0
            .insert(hook_id, erased_hook);
        self.emit::<OnHookAttach<E>>(ctx, &(hook as Arc<dyn TaskHook<E>>))
            .await;
    }

    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        let interested_event_container = self.0.get(E::EVENT_ID)?;

        let entry = interested_event_container.0.get(&TypeId::of::<T>())?;
        let entry = entry.value();

        entry.clone().as_arc_any().downcast::<T>().ok()
    }

    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self, ctx: &TaskHookContext<'_>) {
        let Some(event_category) = self.0.get_mut(E::EVENT_ID) else {
            return;
        };

        let Some((_, hook)) = event_category.0.remove(&TypeId::of::<T>()) else {
            return;
        };

        let erased = hook.clone();

        let typed: Arc<T> = match erased.as_arc_any().downcast::<T>() {
            Ok(typed) => typed,
            Err(actual) => panic!(
                "Failed to downcast stored TaskHook to expected concrete type '{}'. Event ID: '{}'. Expected TypeId: {:?}, actual TypeId: {:?}. \
                Ensure the hook stored under this event is of the requested type and there are no type mismatches.",
                std::any::type_name::<T>(),
                E::EVENT_ID,
                TypeId::of::<T>(),
                actual.as_ref().type_id()
            ),
        };

        self.emit::<OnHookDetach<E>>(ctx, &(typed as Arc<dyn TaskHook<E>>))
            .await;
    }

    pub async fn emit<E: TaskHookEvent>(
        &self,
        ctx: &TaskHookContext<'_>,
        payload: &E::Payload<'_>,
    ) {
        if let Some(entry) = self.0.get(E::EVENT_ID) {
            for hook in entry.value().0.iter() {
                hook.value().on_emit(ctx, ErasedPayload::new(payload)).await;
            }
        }
    }
}
