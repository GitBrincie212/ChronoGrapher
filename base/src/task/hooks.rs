use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::utils::macros::{define_event, define_event_group};
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

/*  TODO: Memory leakage is possible when Task is fully dropped (including its erased forms)

    Ensure a mechanism for when there are no references to the Task (including the erased tasks),
    to notify the registry to drop everything linked to the Task.

    One of the problems will be the (TypeId, usize) pair, while usize is known as its the instance ID,
    the TypeId isn't.
*/

pub(crate) static TASKHOOK_REGISTRY: LazyLock<TaskHookContainer> = LazyLock::new(|| TaskHookContainer(DashMap::new()));

/*
    The TaskHook registry use a promotion-based system to reduce unnecessary memory allocations for
    small enough Event -> TaskHook instances, the same idea applies to TaskHookInstances. The reason
    for the existence of TaskHookInstances is to have a history of the instances (it is a queue essentially).
 */

#[derive(Default)]
pub(crate) enum TaskHookInstances {
    #[default]
    Empty,
    Single(&'static dyn ErasedTaskHook),
    Multiple(Vec<&'static dyn ErasedTaskHook>)
}

impl TaskHookInstances {
    #[inline(always)]
    fn push(&mut self, hook: &'static dyn ErasedTaskHook) {
        match self {
            TaskHookInstances::Empty => {
                *self = TaskHookInstances::Single(hook)
            },
            TaskHookInstances::Single(prev_hook) => {
                *self = TaskHookInstances::Multiple(vec![*prev_hook, hook])
            }

            TaskHookInstances::Multiple(hooks) => {
                hooks.push(hook);
            }
        }
    }

    #[inline(always)]
    fn get(&self) -> &'static dyn ErasedTaskHook {
        match self {
            TaskHookInstances::Empty => { unreachable!() },
            TaskHookInstances::Single(prev_hook) => { *prev_hook }
            TaskHookInstances::Multiple(hooks) => { unsafe { *hooks.last().unwrap_unchecked() } }
        }
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<&'static dyn ErasedTaskHook> {
        match std::mem::take(self) {
            TaskHookInstances::Empty => None,
            TaskHookInstances::Single(instance) => Some(instance),

            TaskHookInstances::Multiple(mut instances) => {
                let val = unsafe { instances.pop().unwrap_unchecked() };
                if instances.len() == 1 {
                    *self = TaskHookInstances::Single(unsafe { instances.pop().unwrap_unchecked() });
                } else {
                    *self = TaskHookInstances::Multiple(instances);
                }
                Some(val)
            }
        }
    }
}

#[derive(Default)]
pub(crate) enum TaskHooksPromotion {
    #[default]
    Empty,
    Single(TypeId, TaskHookInstances),
    Double((TypeId, TaskHookInstances), (TypeId, TaskHookInstances)),
    Triplet((TypeId, TaskHookInstances), (TypeId, TaskHookInstances), (TypeId, TaskHookInstances)),
    Multiple(HashMap<TypeId, TaskHookInstances>)
}

impl TaskHooksPromotion {
    #[inline(always)]
    fn promote(&mut self, hook_id: TypeId, hook: &'static dyn ErasedTaskHook) {
        match self {
            TaskHooksPromotion::Empty => {
                *self = TaskHooksPromotion::Single(hook_id, TaskHookInstances::Single(hook));
            }

            TaskHooksPromotion::Single(prev_id, prev_hook) => {
                if prev_id == &hook_id {
                    prev_hook.push(hook);
                    return;
                }
                let prev_instances = std::mem::take(prev_hook);

                *self = TaskHooksPromotion::Double(
                    (*prev_id, prev_instances),
                    (hook_id, TaskHookInstances::Single(hook))
                );
            }

            TaskHooksPromotion::Double(
                (id1, hooks1),
                (id2, hooks2)
            ) => {
                if id1 == &hook_id {
                    hooks1.push(hook);
                    return;
                } else if id2 == &hook_id {
                    hooks2.push(hook);
                    return;
                }

                let prev_instances1 = std::mem::take(hooks1);
                let prev_instances2 = std::mem::take(hooks2);

                *self = TaskHooksPromotion::Triplet(
                    (*id1, prev_instances1),
                    (*id2, prev_instances2),
                    (hook_id, TaskHookInstances::Single(hook))
                );
            }

            TaskHooksPromotion::Triplet(
                (id1, hooks1),
                (id2, hooks2),
                (id3, hooks3),
            ) => {
                if id1 == &hook_id {
                    hooks1.push(hook);
                    return;
                } else if id2 == &hook_id {
                    hooks2.push(hook);
                    return;
                }  else if id3 == &hook_id {
                    hooks3.push(hook);
                    return;
                }

                let prev_instances1 = std::mem::take(hooks1);
                let prev_instances2 = std::mem::take(hooks2);
                let prev_instances3 = std::mem::take(hooks3);

                let mut map = HashMap::with_capacity(4);
                map.insert(*id1, prev_instances1);
                map.insert(*id2, prev_instances2);
                map.insert(*id3, prev_instances3);
                map.insert(hook_id, TaskHookInstances::Single(hook));
                *self = TaskHooksPromotion::Multiple(map);
            }

            TaskHooksPromotion::Multiple(map) => {
                map.entry(hook_id).or_insert_with(TaskHookInstances::default).push(hook);
            }
        }
    }

    #[inline(always)]
    fn fetch(&self, hook_id: &TypeId) -> Option<&'static dyn ErasedTaskHook> {
        match self {
            TaskHooksPromotion::Single(id, instances) => {
                if *id == *hook_id {return Some(instances.get())}
            }
            TaskHooksPromotion::Double((id1, instances1), (id2, instances2)) => {
                if *id1 == *hook_id {return Some(instances1.get())}
                if *id2 == *hook_id {return Some(instances2.get())}
            }
            TaskHooksPromotion::Triplet(
                (id1, instances1),
                (id2, instances2),
                (id3, instances3)
            ) => {
                if *id1 == *hook_id {return Some(instances1.get())}
                if *id2 == *hook_id {return Some(instances2.get())}
                if *id3 == *hook_id {return Some(instances3.get())}
            }
            TaskHooksPromotion::Multiple(vals) => {
                return Some(vals.get(hook_id)?.get());
            }

            _ => {}
        };

        None
    }

    #[inline(always)]
    fn remove(&mut self, hook_id: TypeId) -> Option<&'static dyn ErasedTaskHook> {
        match self {
            TaskHooksPromotion::Double(
                (id1, instances1),
                (id2, instances2)
            ) => {
                if *id1 == hook_id {
                    if let Some(instance) = instances1.pop() {
                        return Some(instance)
                    }

                    let hook2 = std::mem::take(instances2);
                    *self = TaskHooksPromotion::Single(*id2, hook2);
                } else if *id2 == hook_id {
                    if let Some(instance) = instances2.pop() {
                        return Some(instance)
                    }

                    let hook1 = std::mem::take(instances1);
                    *self = TaskHooksPromotion::Single(*id1, hook1);
                }

                None
            }
            TaskHooksPromotion::Triplet(
                (id1, instances1),
                (id2, instances2),
                (id3, instances3)
            ) => {
                if *id1 == hook_id {
                    if let Some(instance) = instances1.pop() {
                        return Some(instance)
                    }

                    let hooks2 = std::mem::take(instances2);
                    let hooks3 = std::mem::take(instances3);
                    *self = TaskHooksPromotion::Double(
                        (*id2, hooks2),
                        (*id3, hooks3)
                    );
                } else if *id2 == hook_id {
                    if let Some(instance) = instances2.pop() {
                        return Some(instance)
                    }

                    let hooks1 = std::mem::take(instances1);
                    let hooks3 = std::mem::take(instances3);
                    *self = TaskHooksPromotion::Double(
                        (*id1, hooks1),
                        (*id3, hooks3)
                    );

                } else if *id3 == hook_id {
                    if let Some(instance) = instances3.pop() {
                        return Some(instance)
                    }

                    let hooks1 = std::mem::take(instances1);
                    let hooks2 = std::mem::take(instances2);
                    *self = TaskHooksPromotion::Double(
                        (*id1, hooks1),
                        (*id2, hooks2)
                    );
                }

                None
            }

            TaskHooksPromotion::Multiple(map) => {
                let instance = map.remove(&hook_id)?.pop();

                if map.len() == 3 {
                    let mut drained = map.drain();
                    let (id1, hooks1) = unsafe { drained.next().unwrap_unchecked() };
                    let (id2, hooks2) = unsafe { drained.next().unwrap_unchecked() };
                    let (id3, hooks3) = unsafe { drained.next().unwrap_unchecked() };
                    drop(drained);
                    *self = TaskHooksPromotion::Triplet(
                        (id1, hooks1),
                        (id2, hooks2),
                        (id3, hooks3)
                    );
                }

                instance
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
    pub fn attach<E: TaskHookEvent, T: TaskHook<E>>(
        &self,
        ctx: &TaskHookContext<'_>,
        hook: Arc<T>,
    ) -> impl Future<Output = ()> + Send {
        let hook_id = TypeId::of::<T>();
        let erased_hook: &'static dyn ErasedTaskHook =
            Box::leak(Box::new(ErasedTaskHookWrapper::<E>::new(hook.clone())));

        self.0.entry((TypeId::of::<E>(), ctx.instance_id))
            .or_insert(TaskHooksPromotion::Empty)
            .promote(hook_id, erased_hook);

        async move {
            self.emit::<OnHookAttach<E>>(ctx, &(hook.as_ref() as &dyn TaskHook<E>)).await;
        }
    }

    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self, instance_id: usize) -> Option<Arc<T>> {
        let interested_event_container = self.0.get(&(TypeId::of::<E>(), instance_id))?;

        let entry = interested_event_container.fetch(&TypeId::of::<T>())?;

        entry.as_any().downcast::<T>().ok()
    }

    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self, ctx: &TaskHookContext<'_>) {
        let Some(mut event_category) = self.0.get_mut(&(TypeId::of::<E>(), ctx.instance_id)) else {
            return;
        };

        let Some(hook) = event_category.remove(TypeId::of::<T>()) else {
            return;
        };

        let typed: Arc<T> = match hook.as_any().downcast::<T>() {
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

        let wrapper_ptr = hook as *const dyn ErasedTaskHook as *const ();
        let wrapper_ptr = wrapper_ptr as *mut ErasedTaskHookWrapper<E>;
        let wrapper_box = unsafe { Box::from_raw(wrapper_ptr) };
        drop(wrapper_box);

        self.emit::<OnHookDetach<E>>(ctx, &(typed.as_ref() as &dyn TaskHook<E>)).await;
    }

    pub async fn emit<E: TaskHookEvent>(
        &self,
        ctx: &TaskHookContext<'_>,
        payload: &E::Payload<'_>,
    ) {
        if let Some(entry) = self.0.get(&(TypeId::of::<E>(), ctx.instance_id)) {
            let val = entry.value();
            match val {
                TaskHooksPromotion::Empty => {}
                TaskHooksPromotion::Single(_, hook) => {
                    let hook = hook.get();
                    drop(entry);
                    hook.on_emit(ctx, &payload).await;
                }
                TaskHooksPromotion::Double(
                    (_, hook1),
                    (_, hook2)
                ) => {
                    let hook1 = hook1.get();
                    let hook2 = hook2.get();
                    drop(entry);
                    hook1.on_emit(ctx, &payload).await;
                    hook2.on_emit(ctx, &payload).await;
                }
                TaskHooksPromotion::Triplet(
                    (_, hook1),
                    (_, hook2),
                    (_, hook3)
                ) => {
                    let hook1 = hook1.get();
                    let hook2 = hook2.get();
                    let hook3 = hook3.get();
                    drop(entry);
                    hook1.on_emit(ctx, &payload).await;
                    hook2.on_emit(ctx, &payload).await;
                    hook3.on_emit(ctx, &payload).await;
                }
                TaskHooksPromotion::Multiple(vals) => {
                    let mut instances = Vec::with_capacity(vals.len());
                    for hook in vals.values() {
                        instances.push(hook.get());
                    }

                    drop(entry);

                    for hook in instances {
                        hook.on_emit(ctx, &payload).await;
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
pub trait TaskHook<E: TaskHookEvent>: Any + Send + Sync {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &E::Payload<'_>) {}
}

pub trait StaleTaskHook: Any + Send + Sync {}

impl<T: TaskHook<()>> StaleTaskHook for T {}
impl<T: Any + Send + Sync> TaskHook<()> for T {}

#[async_trait]
pub trait TaskHookErased: Send + Sync {
    async fn call_erased(
        &self,
        ctx: &TaskHookContext,
        payload: &dyn Any + Send + Sync,
    );
}

#[async_trait]
impl<T, E> TaskHookErased for T
where
    T: TaskHook<E> + Send + Sync,
    E: TaskHookEvent,
{
    async fn call_erased(
        &self,
        ctx: &TaskHookContext,
        payload: &dyn Any + Send + Sync,
    ) {
        let payload = unsafe {
            &*(payload as *const (dyn Any + Send + Sync) as *const <E as TaskHookEvent>::Payload<'_>)
        };

        self.on_event(ctx, *payload).await;
    }
}

/*
#[async_trait]
impl<E: TaskHookEvent> ErasedTaskHook for dyn TaskHook<E> {
    async fn on_emit(&self, ctx: &TaskHookContext, payload: &(dyn Send + Sync)) {
        let payload = unsafe {
            &*(payload as *const (dyn Send + Sync) as *const &<E as TaskHookEvent>::Payload<'_>)
        };

        self.on_event(ctx, payload).await;
    }
}
*/

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
            type Payload<'a> = &'a dyn TaskHook<E> where Self: 'a;
        }
    };
}

define_hook_event!(OnHookAttach);

define_hook_event!(OnHookDetach);

pub trait TaskHookLifecycleEvents<'a, E: TaskHookEvent>:
    TaskHookEvent<Payload<'a> = &'a dyn TaskHook<E>>
{
}

impl<'a, E: TaskHookEvent> TaskHookLifecycleEvents<'a, E> for OnHookAttach<E> {}
impl<'a, E: TaskHookEvent> TaskHookLifecycleEvents<'a, E> for OnHookDetach<E> {}

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
