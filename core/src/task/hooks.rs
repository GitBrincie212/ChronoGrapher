use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};

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

pub(crate) static TASKHOOK_REGISTRY: LazyLock<TaskHookContainer> = LazyLock::new(|| TaskHookContainer(DashMap::new()));

#[derive(Default)]
pub(crate) enum TaskHooksPromotion {
    #[default]
    Empty,
    Single(TypeId, Arc<dyn ErasedTaskHook>),
    Double((TypeId, Arc<dyn ErasedTaskHook>), (TypeId, Arc<dyn ErasedTaskHook>)),
    Triplet((TypeId, Arc<dyn ErasedTaskHook>), (TypeId, Arc<dyn ErasedTaskHook>), (TypeId, Arc<dyn ErasedTaskHook>)),
    Multiple(HashMap<TypeId, Arc<dyn ErasedTaskHook>>)
}

impl TaskHooksPromotion {
    fn promote(&mut self, hook_id: TypeId, hook: Arc<dyn ErasedTaskHook>) {
        match self {
            TaskHooksPromotion::Empty => {
                *self = TaskHooksPromotion::Single(hook_id, hook);
            }

            TaskHooksPromotion::Single(prev_id, prev_hook) => {
                *self = TaskHooksPromotion::Double((prev_id.clone(), prev_hook.clone()), (hook_id, hook));
            }

            TaskHooksPromotion::Double(prev_pair1, prev_pair2) => {
                *self = TaskHooksPromotion::Triplet(
                    prev_pair1.clone(),
                    prev_pair2.clone(),
                    (hook_id, hook)
                );
            }
            TaskHooksPromotion::Triplet(prev_pair1, prev_pair2, prev_pair3) => {
                let mut map = HashMap::with_capacity(4);
                map.insert(prev_pair1.0, prev_pair1.1.clone());
                map.insert(prev_pair2.0, prev_pair2.1.clone());
                map.insert(prev_pair3.0, prev_pair3.1.clone());
                map.insert(hook_id, hook);
                *self = TaskHooksPromotion::Multiple(map);
            }
            TaskHooksPromotion::Multiple(map) => {
                map.insert(hook_id, hook);
            }
        }
    }

    fn fetch(&self, hook_id: &TypeId) -> Option<&Arc<dyn ErasedTaskHook>> {
        match self {
            TaskHooksPromotion::Single(id, hook) => {
                if *id == *hook_id {return Some(hook)}
            }
            TaskHooksPromotion::Double((id1, hook1), (id2, hook2)) => {
                if *id1 == *hook_id {return Some(hook1)}
                if *id2 == *hook_id {return Some(hook2)}
            }
            TaskHooksPromotion::Triplet(
                (id1, hook1),
                (id2, hook2),
                (id3, hook3)
            ) => {
                if *id1 == *hook_id {return Some(hook1)}
                if *id2 == *hook_id {return Some(hook2)}
                if *id3 == *hook_id {return Some(hook3)}
            }
            TaskHooksPromotion::Multiple(vals) => {
                return vals.get(hook_id);
            }

            _ => {}
        };

        None
    }

    fn remove(&mut self, hook_id: &TypeId) -> Option<Arc<dyn ErasedTaskHook>> {
        match self {
            TaskHooksPromotion::Double(prev_pair1, prev_pair2) => {
                if prev_pair1.0 == *hook_id {
                    let hook1 = prev_pair1.1.clone();
                    *self = TaskHooksPromotion::Single(prev_pair2.0.clone(), prev_pair2.1.clone());
                    return Some(hook1);
                } else if prev_pair2.0 == *hook_id {
                    let hook2 = prev_pair2.1.clone();
                    *self = TaskHooksPromotion::Single(prev_pair1.0.clone(), prev_pair1.1.clone());
                    return Some(hook2);
                }

                None
            }
            TaskHooksPromotion::Triplet(prev_pair1, prev_pair2, prev_pair3) => {
                if prev_pair1.0 == *hook_id {
                    let hook1 = prev_pair1.1.clone();
                    *self = TaskHooksPromotion::Double(prev_pair2.clone(), prev_pair3.clone());
                    return Some(hook1);
                } else if prev_pair2.0 == *hook_id {
                    let hook2 = prev_pair2.1.clone();
                    *self = TaskHooksPromotion::Double(prev_pair1.clone(), prev_pair3.clone());
                    return Some(hook2);
                } else if prev_pair3.0 == *hook_id {
                    let hook3 = prev_pair3.1.clone();
                    *self = TaskHooksPromotion::Double(prev_pair1.clone(), prev_pair2.clone());
                    return Some(hook3);
                }

                None
            }

            TaskHooksPromotion::Multiple(map) => {
                map.remove(hook_id)
            }

            _ => {
                *self = TaskHooksPromotion::Empty;
                None
            }
        }
    }
}

pub(crate) struct TaskHookContainer(pub DashMap<(TypeId, usize), TaskHooksPromotion>);

impl TaskHookContainer {
    pub async fn attach<E: TaskHookEvent, T: TaskHook<E>>(
        &self,
        ctx: &TaskHookContext<'_>,
        hook: Arc<T>,
    ) {
        let hook_id = TypeId::of::<T>();
        let erased_hook: Arc<dyn ErasedTaskHook> =
            Arc::new(ErasedTaskHookWrapper::<E>::new(hook.clone()));

        self.0.entry((TypeId::of::<E>(), ctx.instance_id))
            .or_insert(TaskHooksPromotion::Empty)
            .promote(hook_id, erased_hook.clone());

        self.emit::<OnHookAttach<E>>(ctx, &(hook as Arc<dyn TaskHook<E>>))
            .await;
    }

    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self, instance_id: usize) -> Option<Arc<T>> {
        let interested_event_container = self.0.get(&(TypeId::of::<E>(), instance_id))?;

        let entry = interested_event_container.fetch(&TypeId::of::<T>())?;

        entry.clone().as_arc_any().downcast::<T>().ok()
    }

    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self, ctx: &TaskHookContext<'_>) {
        let Some(mut event_category) = self.0.get_mut(&(TypeId::of::<E>(), ctx.instance_id)) else {
            return;
        };

        let Some(hook) = event_category.remove(&TypeId::of::<T>()) else {
            return;
        };

        let erased = hook.clone();

        let typed: Arc<T> = match erased.as_arc_any().downcast::<T>() {
            Ok(typed) => typed,
            Err(actual) => panic!(
                "Failed to downcast stored TaskHook to expected concrete type '{}'. Event ID: '{}'. Expected TypeId: {:?}, actual TypeId: {:?}. \
                Ensure the hook stored under this event is of the requested type and there are no type mismatches.",
                std::any::type_name::<T>(),
                std::any::type_name::<E>(),
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
        if let Some(entry) = self.0.get(&(TypeId::of::<E>(), ctx.instance_id)) {
            match entry.value() {
                TaskHooksPromotion::Empty => {}
                TaskHooksPromotion::Single(_, hook) => {
                    hook.on_emit(ctx, ErasedPayload::new(payload)).await;
                }
                TaskHooksPromotion::Double(
                    (_, hook1),
                    (_, hook2)
                ) => {
                    hook1.on_emit(ctx, ErasedPayload::new(payload)).await;
                    hook2.on_emit(ctx, ErasedPayload::new(payload)).await;
                }
                TaskHooksPromotion::Triplet(
                    (_, hook1),
                    (_, hook2),
                    (_, hook3)
                ) => {
                    hook1.on_emit(ctx, ErasedPayload::new(payload)).await;
                    hook2.on_emit(ctx, ErasedPayload::new(payload)).await;
                    hook3.on_emit(ctx, ErasedPayload::new(payload)).await;
                }
                TaskHooksPromotion::Multiple(vals) => {
                    for hook in vals.values() {
                        hook.on_emit(ctx, ErasedPayload::new(payload)).await;
                    }
                }
            }
        }
    }
}

pub trait TaskHookEvent: Send + Sync + Default + 'static {
    type Payload<'a>: Send + Sync
    where
        Self: 'a;
}

pub enum NonEmittable {}

impl TaskHookEvent for () {
    type Payload<'a>
        = NonEmittable
    where
        Self: 'a;
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

#[derive(Clone)]
pub struct TaskHookContext<'a> {
    pub(crate) depth: u64,
    pub(crate) instance_id: usize,
    pub(crate) frame: &'a dyn ErasedTaskFrame,
}

impl<'a> TaskHookContext<'a> {
    pub fn depth(&self) -> u64 {
        self.depth
    }

    pub fn frame(&self) -> &dyn ErasedTaskFrame {
        self.frame
    }

    pub async fn emit<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) {
        TASKHOOK_REGISTRY.emit::<E>(self, payload).await;
    }

    pub async fn attach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self, hook: Arc<T>) {
        TASKHOOK_REGISTRY.attach::<E, T>(self, hook).await;
    }

    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        TASKHOOK_REGISTRY.detach::<E, T>(self).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<E, T>(self.instance_id)
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
