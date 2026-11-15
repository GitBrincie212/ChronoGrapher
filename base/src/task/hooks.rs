use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::task::{Sealed, TaskHookLayer};
use crate::utils::macros::{define_event, define_event_group};
use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock, OnceLock};

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
    pub use crate::task::frames::OnFallback;
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

pub(crate) static TASKHOOK_REGISTRY: LazyLock<TaskHookContainer> =
    LazyLock::new(|| TaskHookContainer(DashMap::new()));

/*
   The TaskHook registry use a promotion-based system to reduce unnecessary memory allocations for
   small enough Event -> TaskHook instances, the same idea applies to TaskHookInstances. The reason
   for the existence of TaskHookInstances is to have a history of the instances (it is a queue essentially).
*/

#[derive(Default)]
enum HookStack<E: TaskHookEvent> {
    #[default]
    Empty,
    Single(Arc<RegisteredHook<E>>),
    Multiple(Vec<Arc<RegisteredHook<E>>>),
}

impl<E: TaskHookEvent> HookStack<E> {
    #[inline(always)]
    fn push(&mut self, hook: Arc<RegisteredHook<E>>) {
        match self {
            HookStack::Empty => *self = HookStack::Single(hook),
            HookStack::Single(prev_hook) => {
                *self = HookStack::Multiple(vec![prev_hook.clone(), hook])
            }

            HookStack::Multiple(hooks) => {
                hooks.push(hook);
            }
        }
    }

    #[inline(always)]
    fn listener(&self) -> Option<Arc<RegisteredHook<E>>> {
        match self {
            Self::Empty => None,
            Self::Single(hook) => Some(hook.clone()),
            Self::Multiple(hooks) => hooks.last().cloned(),
        }
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Arc<RegisteredHook<E>>> {
        match std::mem::take(self) {
            HookStack::Empty => None,
            HookStack::Single(instance) => Some(instance),

            HookStack::Multiple(mut instances) => {
                let value = instances.pop();
                *self = match instances.len() {
                    0 => Self::Empty,
                    1 => Self::Single(instances.remove(0)),
                    _ => Self::Multiple(instances),
                };
                value
            }
        }
    }
}

#[derive(Default)]
enum EventHooks<E: TaskHookEvent> {
    #[default]
    Empty,
    Single(TypeId, HookStack<E>),
    Double((TypeId, HookStack<E>), (TypeId, HookStack<E>)),
    Triplet(
        (TypeId, HookStack<E>),
        (TypeId, HookStack<E>),
        (TypeId, HookStack<E>),
    ),
    Multiple(HashMap<TypeId, HookStack<E>>),
}

impl<E: TaskHookEvent> EventHooks<E> {
    #[inline(always)]
    fn promote(&mut self, hook_id: TypeId, hook: Arc<RegisteredHook<E>>) {
        match self {
            EventHooks::Empty => {
                *self = EventHooks::Single(hook_id, HookStack::Single(hook));
            }

            EventHooks::Single(prev_id, prev_hook) => {
                if prev_id == &hook_id {
                    prev_hook.push(hook);
                    return;
                }
                let prev_instances = std::mem::take(prev_hook);

                *self = EventHooks::Double(
                    (*prev_id, prev_instances),
                    (hook_id, HookStack::Single(hook)),
                );
            }

            EventHooks::Double((id1, hooks1), (id2, hooks2)) => {
                if id1 == &hook_id {
                    hooks1.push(hook);
                    return;
                } else if id2 == &hook_id {
                    hooks2.push(hook);
                    return;
                }

                let prev_instances1 = std::mem::take(hooks1);
                let prev_instances2 = std::mem::take(hooks2);

                *self = EventHooks::Triplet(
                    (*id1, prev_instances1),
                    (*id2, prev_instances2),
                    (hook_id, HookStack::Single(hook)),
                );
            }

            EventHooks::Triplet((id1, hooks1), (id2, hooks2), (id3, hooks3)) => {
                if id1 == &hook_id {
                    hooks1.push(hook);
                    return;
                } else if id2 == &hook_id {
                    hooks2.push(hook);
                    return;
                } else if id3 == &hook_id {
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
                map.insert(hook_id, HookStack::Single(hook));
                *self = EventHooks::Multiple(map);
            }

            EventHooks::Multiple(map) => {
                map.entry(hook_id)
                    .or_insert_with(HookStack::default)
                    .push(hook);
            }
        }
    }

    #[inline(always)]
    fn fetch(&self, hook_id: &TypeId) -> Option<&HookStack<E>> {
        match self {
            EventHooks::Single(id, instances) if *id == *hook_id => {
                return Some(instances);
            }
            EventHooks::Double((id1, instances1), (id2, instances2)) => {
                if *id1 == *hook_id {
                    return Some(instances1);
                }
                if *id2 == *hook_id {
                    return Some(instances2);
                }
            }
            EventHooks::Triplet((id1, instances1), (id2, instances2), (id3, instances3)) => {
                if *id1 == *hook_id {
                    return Some(instances1);
                }
                if *id2 == *hook_id {
                    return Some(instances2);
                }
                if *id3 == *hook_id {
                    return Some(instances3);
                }
            }
            EventHooks::Multiple(vals) => {
                return vals.get(hook_id);
            }

            _ => {}
        };

        None
    }

    #[inline(always)]
    fn remove(&mut self, hook_id: TypeId) -> Option<Arc<RegisteredHook<E>>> {
        match self {
            Self::Single(id, instances) if *id == hook_id => instances.pop(),
            Self::Double((id1, first), (id2, second)) => {
                if *id1 == hook_id {
                    first.pop()
                } else if *id2 == hook_id {
                    second.pop()
                } else {
                    None
                }
            }
            Self::Triplet((id1, first), (id2, second), (id3, third)) => {
                if *id1 == hook_id {
                    first.pop()
                } else if *id2 == hook_id {
                    second.pop()
                } else if *id3 == hook_id {
                    third.pop()
                } else {
                    None
                }
            }
            Self::Multiple(map) => {
                let stack = map.get_mut(&hook_id)?;
                let removed = stack.pop();
                if matches!(stack, HookStack::Empty) {
                    map.remove(&hook_id);
                }
                removed
            }
            _ => None,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Single(_, stack) => matches!(stack, HookStack::Empty),
            Self::Double((_, first), (_, second)) => {
                matches!((first, second), (HookStack::Empty, HookStack::Empty))
            }
            Self::Triplet((_, first), (_, second), (_, third)) => matches!(
                (first, second, third),
                (HookStack::Empty, HookStack::Empty, HookStack::Empty)
            ),
            Self::Multiple(map) => map.is_empty(),
        }
    }

    fn snapshot(&self) -> Vec<Arc<RegisteredHook<E>>> {
        match self {
            Self::Empty => Vec::new(),
            Self::Single(_, hook) => hook.listener().into_iter().collect(),
            Self::Double((_, first), (_, second)) => [first.listener(), second.listener()]
                .into_iter()
                .flatten()
                .collect(),
            Self::Triplet((_, first), (_, second), (_, third)) => {
                [first.listener(), second.listener(), third.listener()]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            Self::Multiple(map) => map.values().filter_map(HookStack::listener).collect(),
        }
    }
}

// Erase the collection, not the payload. Each bucket stores hooks for exactly one event type.
type TaskEventHooks = DashMap<TypeId, Box<dyn Any + Send + Sync>>;

pub(crate) struct TaskHookContainer(DashMap<usize, OnceLock<Arc<TaskEventHooks>>>);

// This guard moves with Task::into_erased, so type erasure never unregisters a live task.
pub(crate) struct TaskHookOwner(usize);

impl TaskHookOwner {
    pub(crate) fn new(instance_id: usize) -> Self {
        TASKHOOK_REGISTRY.0.insert(instance_id, OnceLock::new());
        Self(instance_id)
    }
}

impl Drop for TaskHookOwner {
    fn drop(&mut self) {
        // remove returns owned data. User hook destructors run after the registry lock is released.
        let removed = TASKHOOK_REGISTRY.0.remove(&self.0);
        drop(removed);
    }
}

impl TaskHookContainer {
    fn events(&self, instance_id: usize) -> Option<Arc<TaskEventHooks>> {
        self.0.get(&instance_id)?.get().cloned()
    }

    pub async fn attach<E: TaskHookEvent>(
        &self,
        ctx: &TaskHookContext,
        hook: Arc<impl TaskHook<E>>,
    ) {
        let Some(events) = self
            .0
            .get(&ctx.0)
            .map(|entry| entry.get_or_init(|| Arc::new(DashMap::new())).clone())
        else {
            return;
        };
        let hook_id = hook.as_ref().type_id();
        let registered = Arc::new(RegisteredHook::<E>::new(hook.clone()));
        events
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventHooks::<E>::Empty))
            .downcast_mut::<EventHooks<E>>()
            .expect("event type must match its registry key")
            .promote(hook_id, registered);
        drop(events);

        self.emit::<OnHookAttach<E>>(ctx, &(hook.as_ref() as &dyn TaskHook<E>))
            .await;
    }

    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self, instance_id: usize) -> Option<Arc<T>> {
        let events = self.events(instance_id)?;
        let category = events.get(&TypeId::of::<E>())?;
        let concrete = category
            .downcast_ref::<EventHooks<E>>()?
            .fetch(&TypeId::of::<T>())?
            .listener()?
            .concrete
            .clone();
        drop(category);
        concrete.downcast::<T>().ok()
    }

    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self, ctx: &TaskHookContext) {
        let Some(events) = self.events(ctx.0) else {
            return;
        };
        let hook = {
            let Entry::Occupied(mut category) = events.entry(TypeId::of::<E>()) else {
                return;
            };
            let Some(hooks) = category.get_mut().downcast_mut::<EventHooks<E>>() else {
                return;
            };
            let removed = hooks.remove(TypeId::of::<T>());
            if hooks.is_empty() {
                category.remove();
            }
            removed
        };
        drop(events);
        let Some(hook) = hook else {
            return;
        };
        let typed = hook
            .concrete
            .clone()
            .downcast::<T>()
            .expect("registered hook type must match");
        drop(hook);
        self.emit::<OnHookDetach<E>>(ctx, &(typed.as_ref() as &dyn TaskHook<E>))
            .await;
    }

    pub async fn emit<E: TaskHookEvent>(&self, ctx: &TaskHookContext, payload: &E::Payload<'_>) {
        let Some(events) = self.events(ctx.0) else {
            return;
        };
        let hooks = events
            .get(&TypeId::of::<E>())
            .and_then(|entry| {
                entry
                    .downcast_ref::<EventHooks<E>>()
                    .map(EventHooks::snapshot)
            })
            .unwrap_or_default();
        drop(events);
        // The snapshot owns the live listeners until dispatch completes.
        for hook in hooks {
            hook.listener.on_event(ctx, payload).await;
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
/// The task retains the listener until detachment or task destruction. Use a
/// `Weak` back-reference to an `Arc<Task<_>>` to avoid a reference cycle.
pub trait TaskHook<E: TaskHookEvent>: Send + Sync + 'static {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &E::Payload<'_>) {}
}

pub trait NonObserverTaskHook: Send + Sync + 'static {}

#[async_trait]
impl<T: NonObserverTaskHook> TaskHook<()> for T {}

struct RegisteredHook<E: TaskHookEvent> {
    listener: Arc<dyn TaskHook<E>>,
    concrete: Arc<dyn Any + Send + Sync>,
}

impl<E: TaskHookEvent> RegisteredHook<E> {
    fn new<T: TaskHook<E>>(hook: Arc<T>) -> Self {
        Self {
            listener: hook.clone(),
            concrete: hook,
        }
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
            type Payload<'a> = &'a dyn TaskHook<E> where Self: 'a;
        }
    };
}

define_hook_event!(OnHookAttach);

define_hook_event!(OnHookDetach);

pub trait TaskHookLifecycleEvents<E: TaskHookEvent>:
    for<'a> TaskHookEvent<Payload<'a> = &'a dyn TaskHook<E>>
{
}

impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookAttach<E> {}
impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookDetach<E> {}

/// A task-scoped hook context. Hook operations become inactive after the owning task drops.
/// An emission that already captured its hooks may finish after removal.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaskHookContext(pub(crate) usize);

impl TaskHookContext {
    pub async fn emit<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) {
        TASKHOOK_REGISTRY.emit::<E>(self, payload).await;
    }

    pub async fn attach_hook<E: TaskHookEvent>(&self, hook: Arc<impl TaskHook<E>>) {
        TASKHOOK_REGISTRY.attach::<E>(self, hook).await;
    }

    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        TASKHOOK_REGISTRY.detach::<E, T>(self).await;
    }

    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        TASKHOOK_REGISTRY.get::<E, T>(self.0)
    }
}

impl Sealed for TaskHookContext {}

impl TaskHookLayer for TaskHookContext {
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
        self.emit::<EV>(payload)
    }

    fn detach<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> impl Future<Output = ()> + Send {
        self.detach_hook::<EV, T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{NoOperationTaskFrame, Task, TaskScheduleImmediate};

    struct TestHook;

    impl TaskHook<OnTaskStart> for TestHook {}

    #[tokio::test]
    async fn last_detach_releases_the_event_bucket() {
        let task = Task::new(
            NoOperationTaskFrame::<String>::default(),
            TaskScheduleImmediate,
        );
        task.attach_hook::<OnTaskStart>(Arc::new(TestHook)).await;
        task.attach_hook::<OnTaskStart>(Arc::new(TestHook)).await;
        let events = TASKHOOK_REGISTRY.events(task.instance_id).unwrap();
        task.detach_hook::<OnTaskStart, TestHook>().await;
        assert!(events.contains_key(&TypeId::of::<OnTaskStart>()));
        task.detach_hook::<OnTaskStart, TestHook>().await;
        assert!(!events.contains_key(&TypeId::of::<OnTaskStart>()));
        task.attach_hook::<OnTaskStart>(Arc::new(TestHook)).await;
        assert!(task.get_hook::<OnTaskStart, TestHook>().is_some());
    }
}
