use crate::persistence::registries::PERSISTENCE_REGISTRIES;
use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
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
    pub use crate::task::frames::OnTaskFrameSelection;
    pub use crate::task::frames::OnTimeout;
    pub use crate::task::frames::OnTruthyValueEvent;
    pub use crate::task::frames::RetryAttemptEvents;
    pub use crate::task::hooks::OnHookAttach;
    pub use crate::task::hooks::OnHookDetach;
    pub use crate::task::hooks::TaskHookEvent;
    pub use crate::task::hooks::TaskHookLifecycleEvents;
    pub use crate::task::hooks::TaskLifecycleEvents;
} // skipcq: RS-D1001

/// [`TaskHookEvent`] is a trait used for describing [`Task`] or [`TaskFrame`] events for [`TaskHook`]
/// instances to hook into. It is meant to be implemented in the form of a marker struct. It contains
/// a payload type for the event (what kind of information does the event include inside) and
/// a Persistence ID
///
/// There are 4 types of [`TaskHookEvent`], These are:
/// 1. **Hook Lifecycle Events** These are [`OnHookAttach`] and [`OnHookDetach`], they are emitted
/// when a [`TaskHook`] is either attached or detached from a [`TaskHookContainer`]. When either
/// events are triggered, unlike most other implementations of [`TaskHookEvent`], they contain a
/// generic indicating the event they are attached to / detached from
///
/// 2. **Task Lifecycle Events** These are [`OnTaskStart`] and [`OnTaskEnd`], they are simple
/// events that execute on [`Task`] when it starts and when it ends respectively. Nothing unique
/// about them, other than that they live in [`Task`]
///
/// 3. **TaskFrame Events** These have to do with [`TaskFrame`] types. They are emitted
/// from [`TaskFrame`], other than that they have nothing else unique about them
///
/// 4. **TaskHook Event Groups** These are typically traits, their goal is to group relevant events
/// making it possible for TaskHooks to run the same code in a group of events (without boilerplate).
/// They cannot not be registered onto TaskHooksContainer and they are part of the TaskHook implementation
/// phase
///
/// # Trait Implementation(s)
/// There are multiple implementations for [`TaskHookEvent`] present in ChronoGrapher, almost
/// all [`TaskFrame`] include at least one relevant [`TaskHookEvent`], a list of standalone events are:
/// - [`OnHookAttach`] - Triggered when a [`TaskHook`] is attached
/// - [`OnHookDetach`] - Triggered when a [`TaskHook`] is detached
/// - [`OnTaskStart`] - Triggered when a [`Task`] starts execution
/// - [`OnTaskEnd`] - Triggered when a [`Task`] ends its execution
/// - [`OnTruthyValueEvent`] - Triggered when a [`ConditionalFramePredicate`] returns true
/// - [`OnFalseyValueEvent`] - Triggered when a [`ConditionalFramePredicate`] returns false
/// - [`OnTimeout`] - Triggered when a timeout occurs in [`TimeoutTaskFrame`]
/// - [`OnRetryAttemptStart`] - Triggered when a retry is attempted in [`RetriableTaskFrame`]
/// - [`OnRetryAttemptEnd`] - Triggered when a retry attempt is finished in [`RetriableTaskFrame`]
/// - [`OnDelayStart`] - Triggers when the idling process starts in [`DelayTaskFrame`]
/// - [`OnDelayEnd`] - Triggers when the idling process ends in [`DelayTaskFrame`]
/// - [`OnChildTaskFrameStart`] - Triggers when a wrapped child [`TaskFrame`] from
/// [`SequentialTaskFrame`] / [`ParallelTaskFrame`] starts
/// - [`OnChildTaskFrameEnd`] - Triggers when a wrapped child [`TaskFrame`] from
/// [`SequentialTaskFrame`] / [`ParallelTaskFrame`] ends
/// - [`OnTaskFrameSelection`] - Triggers when a [`TaskFrame`] is selected from [`SelectTaskFrame`]
/// - [`OnFallbackEvent`] - Triggers when the primary [`TaskFrame`] fails in [`FallbackTaskFrame`]
/// - [`OnDependencyValidation`] - Triggers when a [`TaskDependency`] is validated in [`DependencyTaskFrame`]
///
/// It should also be noted that ``()`` implements the [`TaskHookEvent`] trait as well. The intention
/// of this is to indicate a [`TaskHook`] is not meant to listen to any event (while still implementing
/// the [`TaskHook`] trait). As such, ``()`` cannot be emitted as an event via [`TaskHookContainer::emit`]
/// or similar method aliases. For readability, when implementing ``TaskHook<()>``, one can use
/// [`NonObserverTaskHook`] trait to avoid boilerplate code and make it more concise
///
/// # Supertrait(s)
/// When implementing the [`TaskHookEvent`] trait, one has to supply an implementation for the
/// [`Default`] trait from core Rust (since this system requires to instantiate an instance of the
/// event without caring about the data)
///
/// # Object Safety
/// [`TaskHookEvent`] is **NOT** object safe, due to the fact it uses an associated type
/// and a constant. The reason as to why it is an associated type and not a generic is to
/// always enforce one kind of payload
///
/// # See Also
/// - [`TaskHook`]
/// - [`Task`]
/// - [`TaskFrame`]
/// - [`TaskHookContainer`]
/// - [`OnHookAttach`]
/// - [`OnHookDetach`]
/// - [`OnTaskStart`]
/// - [`OnTaskEnd`]
/// - [`OnTruthyValueEvent`]
/// - [`OnFalseyValueEvent`]
/// - [`OnTimeout`]
/// - [`OnRetryAttemptStart`]
/// - [`OnRetryAttemptEnd`]
/// - [`OnDelayStart`]
/// - [`OnDelayEnd`]
/// - [`OnChildTaskFrameStart`]
/// - [`OnChildTaskFrameEnd`]
/// - [`OnTaskFrameSelection`]
/// - [`OnFallbackEvent`]
/// - [`OnDependencyValidation`]
pub trait TaskHookEvent:
    Send + Sync + Default + 'static + Serialize + for<'de> Deserialize<'de>
{
    type Payload: Send + Sync;
    const PERSISTENCE_ID: &'static str;
}

pub enum NonEmittable {}

impl TaskHookEvent for () {
    type Payload = NonEmittable;
    const PERSISTENCE_ID: &'static str = "";
}

impl<E: TaskHookEvent> PersistenceObject for E {
    const PERSISTENCE_ID: &'static str = Self::PERSISTENCE_ID;

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}

/// [`TaskHook`] is a trait for defining a task hook, task hooks listens to events emitted by the
/// [`TaskFrame`] chain and the [`Task`]'s lifecycle, executing code appropriately. [`TaskFrame`]
/// can work with the extensions during execution and emit events for the extensions to listen. While
/// this system works with [`TaskFrame`], it doesn't know the full [`TaskFrame`] chain structure, making
/// it a flexible system for:
///
/// 1. **Observability / Side Effects** Whenever you want to specifically observe behavior or trigger
/// side effects depending on an event being triggered, for example monitoring, where you want to keep
/// track of the number of timeouts, retries... etc. Where the way the [`TaskFrame`] chain is composed
/// is not a factor to consider
///
/// 2. **State Management** Since [`TaskFrame`] knows all extensions hooked for this [`Task`] its
/// running, it can interact with one or multiple extension(s), reading/writing state. This can be
/// useful for global state between multiple [`Task`] instances or some local state
///
/// A [`TaskHook`] can work with one or more other task hook(s). Making it possible to
/// integrate multiple systems in a unified way, while also splitting concerns. [`TaskHook`]
/// can also be injected from a [`TaskFrame`] onto the [`Task`]
///
/// # Object Safety
/// [`TaskHook`] is object safe as seen in [`TaskHookContainer`]'s source code
///
/// # See Also
/// - [`Task`]
/// - [`TaskFrame`]
#[async_trait]
pub trait TaskHook<E: TaskHookEvent>: Send + Sync + 'static {
    async fn on_event(&self, event: E, ctx: &TaskContext, payload: &E::Payload);
}

/// [`NonObserverTaskHook`] is an alias for [`TaskHook`] where the event type is
/// ``()`` (i.e. No event). This is the same as doing:
/// ```ignore
/// #[async_trait]
/// impl TaskHook<()> for T {
///     async fn on_event(
///         &self,
///         _event: (),
///         _ctx: Arc<TaskContext>,
///         _payload: &<() as TaskHookEvent>::Payload)
///     {}
/// }
/// ```
/// The only purpose it has, is to just serves to save boilerplate hustle and separate an
/// observer task hook versus a non observer task hook. Refer to the documentation for
/// [`TaskHook`], [`TaskHookContainer`] and [`TaskHookEvent`] to learn more about the task hook system
///
/// # See Also
/// - [`TaskHook`]
/// - [`TaskHookEvent`]
/// - [`TaskHookContainer`]
pub trait NonObserverTaskHook: Send + Sync + 'static {}

#[async_trait]
impl<T: NonObserverTaskHook> TaskHook<()> for T {
    async fn on_event(
        &self,
        _event: (),
        _ctx: &TaskContext,
        _payload: &<() as TaskHookEvent>::Payload,
    ) {
    }
}

#[derive(Clone)]
pub(crate) struct ErasedTaskHookWrapper<E: TaskHookEvent>(
    pub(crate) Arc<dyn TaskHook<E>>,
    pub(crate) PhantomData<E>,
);

#[async_trait]
pub trait ErasedTaskHook: Send + Sync {
    async fn on_emit(&self, ctx: &TaskContext, payload: &(dyn Any + Send + Sync));
    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

#[async_trait]
impl<E: TaskHookEvent + 'static> ErasedTaskHook for ErasedTaskHookWrapper<E> {
    async fn on_emit(&self, ctx: &TaskContext, payload: &(dyn Any + Send + Sync)) {
        let payload = payload
            .downcast_ref::<E::Payload>()
            .expect("Invalid payload type passed to TaskHook");

        self.0.on_event(E::default(), ctx, payload).await;
    }

    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

define_event!(
    /// [`OnTaskStart`] is an implementation of [`TaskHookEvent`] (a system used closely with [`TaskHook`]).
    /// The concrete payload type of [`OnTaskStart`] is ``()``
    ///
    /// # Constructor(s)
    /// When constructing a [`OnTaskStart`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnTaskStart::default`] or via simply pasting
    /// the struct name ([`OnTaskStart`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnTaskStart`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnTaskStart`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnTaskStart, ()
);

define_event!(
    /// [`OnTaskEnd`] is an implementation of [`TaskHookEvent`] (a system used closely with [`TaskHook`]).
    /// The concrete payload type of [`OnTaskEnd`] is ``()``
    ///
    /// # Constructor(s)
    /// When constructing a [`OnTaskEnd`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnTaskEnd::default`] or via simply pasting
    /// the struct name ([`OnTaskEnd`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnTaskEnd`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnTaskEnd`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnTaskEnd, Option<TaskError>
);

define_event_group!(
    /// [`TaskLifecycleEvents`] is a marker trait, more specifically a [`TaskHookEvent`] group of
    /// [`TaskHookEvent`] (a system used closely with [`TaskHook`]). It contains no common payload type
    ///
    /// # Supertrait(s)
    /// Since it is a [`TaskHookEvent`] group, it requires every descended to implement the [`TaskHookEvent`],
    /// because no common payload type is present, any payload type is accepted
    ///
    /// # Trait Implementation(s)
    /// Currently, two [`TaskHookEvent`] implement the [`TaskLifecycleEvents`] marker trait
    /// (event group). Those being [`OnTaskStart`] and [`OnTaskEnd`]
    ///
    /// # Object Safety
    /// [`TaskLifecycleEvents`] is **NOT** object safe, due to the fact it implements the
    /// [`TaskHookEvent`] which itself is not object safe
    ///
    /// # See Also
    /// - [`OnTaskStart`]
    /// - [`OnTaskEnd`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    TaskLifecycleEvents,
    OnTaskStart, OnTaskEnd
);

macro_rules! define_hook_event {
    ($(#[$($attrs:tt)*])* $name: ident) => {
        $(#[$($attrs)*])*
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
        pub struct $name<E: TaskHookEvent>(PhantomData<E>);

        impl<E: TaskHookEvent> Default for $name<E> {
            fn default() -> Self {
                $name(PhantomData)
            }
        }

        impl<E: TaskHookEvent> TaskHookEvent for $name<E> {
            type Payload = Arc<dyn TaskHook<E>>;
            const PERSISTENCE_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

define_hook_event!(
    /// [`TaskHook`]). The concrete payload type of [`OnHookAttach`] is ``TaskHookEvent<P>``.
    /// Unlike most events, this is generic-based TaskEvent, it has to do with lifecycle of TaskHooks
    ///
    /// # Constructor(s)
    /// When constructing a [`OnHookAttach`] due to the fact this is a marker ``struct``,
    /// making it as such zero-sized, one can either use [`OnHookAttach::default`]
    /// or via simply pasting the struct name ([`OnHookAttach`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnHookAttach`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnHookAttach`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnHookAttach
);

define_hook_event!(
    /// [`OnHookDetach`] is an implementation of [`TaskHookEvent`] (a system used closely with
    /// [`TaskHook`]). The concrete payload type of [`OnHookDetach`] is ``TaskHookEvent<P>``.
    /// Unlike most events, this is generic-based TaskEvent, it has to do with lifecycle of TaskHooks
    ///
    /// # Constructor(s)
    /// When constructing a [`OnHookDetach`] due to the fact this is a marker ``struct``,
    /// making it as such zero-sized, one can either use [`OnHookDetach::default`]
    /// or via simply pasting the struct name ([`OnHookDetach`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnHookDetach`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnHookDetach`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnHookDetach
);

/// [`TaskHookLifecycleEvents`] is a marker trait with a generic ``E`` as a [`TaskHookEvent`],
/// more specifically a [`TaskHookEvent`] group of [`TaskHookEvent`]
/// (a system used closely with [`TaskHook`]). It as payload type the generic ``E``, however
/// unlike most TaskHookEvent groups, as mentioned this one has a generic. If one would like
/// to listen to all types of hook events, then use the following pattern:
/// ```ignore
/// impl<E2: TaskHookEvent, E: TaskHookLifecycleEvents<E2>> TaskHook<E> for ABC {
///     async fn on_event(&self, event: E, ctx: &TaskContext, payload: &E::Payload) {
///         // ...
///     }
/// }
/// ```
///
/// # Supertrait(s)
/// Since it is a [`TaskHookEvent`] group, it requires every descended to implement the [`TaskHookEvent`],
/// and more specifically have the payload type ``E`` generic
///
/// # Trait Implementation(s)
/// Currently, two [`TaskHookEvent`] implement the [`TaskHookLifecycleEvents`] marker trait
/// (event group). Those being [`OnHookAttach`] and [`OnHookDetach`]
///
/// # Object Safety
/// [`TaskHookLifecycleEvents`] is **NOT** object safe, due to the fact it implements the
/// [`TaskHookEvent`] which itself is not object safe
///
/// # See Also
/// - [`OnHookAttach`]
/// - [`OnHookDetach`]
/// - [`TaskHook`]
/// - [`TaskHookEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
pub trait TaskHookLifecycleEvents<E: TaskHookEvent>:
    TaskHookEvent<Payload = Arc<dyn TaskHook<E>>>
{
}
impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookAttach<E> {}
impl<E: TaskHookEvent> TaskHookLifecycleEvents<E> for OnHookDetach<E> {}

pub(crate) enum UnknownErasedTaskHook {
    Persistent {
        id: &'static str,
        hook: Arc<dyn ErasedTaskHook>,
    },
    Ephemeral(Arc<dyn ErasedTaskHook>),
}

impl UnknownErasedTaskHook {
    fn extract(&self) -> &Arc<dyn ErasedTaskHook> {
        match self {
            UnknownErasedTaskHook::Persistent { hook, .. } => hook,
            UnknownErasedTaskHook::Ephemeral(hook) => hook,
        }
    }
}

#[derive(Default)]
pub(crate) struct TaskHookEventCategory(pub DashMap<TypeId, UnknownErasedTaskHook>);

impl Serialize for TaskHookEventCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for entry in self.0.iter() {
            let key = entry.key();
            let (&id, hook) = match entry.value() {
                UnknownErasedTaskHook::Ephemeral(_) => continue,
                UnknownErasedTaskHook::Persistent { hook, id } => (id, hook),
            };
            if let Some(persistent_entry) = PERSISTENCE_REGISTRIES.get_hook_entry(id) {
                map.serialize_key(id)?;

                // hook.clone().as_arc_any(): The value i want to serialize
                // persistent_entry.serialize: The serialization function (erased)
            }
        }
        map.end()
    }
}

/// [`TaskHookContainer`] is a container that hosts one or multiple [`TaskHook`] instance(s)
/// which have subscribed to one or multiple [`TaskHookEvent(s)`]. This system is utilized
/// by [`Task`]. [`TaskHook(s)`] can be attached and detached at any point or even fetched
/// from the corresponding type. This means at any point where one has access to this container
/// which is via [`Task`] or [`TaskFrame`], they can subscribe / unsubscribe hooks to enable
/// more flexibility. [`TaskFrame`] can also check if specific hooks are attached in order to
/// run side effects (currently not utilized by the core)
///
/// [`TaskHookContainer`] is not meant to be created by outside parties. It only lives
/// inside [`Task`]
///
/// # Example
/// ```ignore
/// use chronographer_core::task::{NoOperationTaskFrame, Task, TaskScheduleImmediate};
///
/// let test_task = Task::define(TaskScheduleImmediate, NoOperationTaskFrame);
/// test_task.attach_hook(MY_ABC_TASK_HOOK); // MY_ABC_TASK_HOOK knows it got attached
///
/// // Somewhere else in code
/// let MY_ABC_TASK_HOOK = test_task.get_hook::<MyABCTaskHook>();
/// MY_ABC_TASK_HOOK.do_something(...);
///
/// // Somewhere else in code
/// test_task.detach::<MyABCTaskHook>(); // MY_ABC_TASK_HOOK knows it got detached
/// ```
///
/// # See Also
/// - [`TaskHook`]
/// - [`TaskHookEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
// #[derive(Serialize, Deserialize)]
pub struct TaskHookContainer(pub(crate) DashMap<&'static str, TaskHookEventCategory>);

impl TaskHookContainer {
    /// Attaches an **Ephemeral** [`TaskHook`] onto the container, when attached, the [`TaskHook`]
    /// is alerted via [`OnHookAttach`], in which, it knows the [`TaskHookEvent`] it is
    /// attached to.
    ///
    /// When the program crashes, these TaskHooks do not persist. Depending on the circumstances,
    /// this may not be a wanted behavior, if you can guarantee your TaskHook is persistable, then
    /// [`TaskHookContainer::attach_persistent`] is the ideal method for you
    ///
    /// # Argument(s)
    /// When attaching a [`TaskHook`], one has to supply the [`TaskHook`] instance
    /// to attach. In addition to that, the event has to be supplied as well as a generic
    ///
    /// # See Also
    /// - [`TaskHookContainer`]
    /// - [`TaskHookEvent`]
    /// - [`TaskHook`]
    /// - [`OnHookAttach`]
    pub async fn attach_ephemeral<E: TaskHookEvent>(
        &self,
        ctx: &TaskContext,
        hook: Arc<impl TaskHook<E>>,
    ) {
        let hook_id = hook.type_id();
        let erased_hook: Arc<dyn ErasedTaskHook> =
            Arc::new(ErasedTaskHookWrapper::<E>(hook.clone(), PhantomData));

        self.0
            .entry(E::PERSISTENCE_ID)
            .or_default()
            .0
            .insert(hook_id, UnknownErasedTaskHook::Ephemeral(erased_hook));
        self.emit::<OnHookAttach<E>>(ctx, &(hook as Arc<dyn TaskHook<E>>))
            .await;
    }

    /// Attaches a **Persistent** [`TaskHook`] onto the container, when attached, the [`TaskHook`]
    /// is alerted via [`OnHookAttach`], in which, it knows the [`TaskHookEvent`] it is
    /// attached to.
    ///
    /// When the program crashes, these TaskHooks do persist. Depending on the circumstances,
    /// this may not be a wanted behavior, if you don't want this to be enforced, then
    /// [`TaskHookContainer::attach_ephemeral`] is the ideal method for you
    ///
    /// # Argument(s)
    /// When attaching a [`TaskHook`], one has to supply the [`TaskHook`] instance
    /// to attach. In addition to that, the event has to be supplied as well as a generic
    ///
    /// # See Also
    /// - [`TaskHookContainer`]
    /// - [`TaskHookEvent`]
    /// - [`TaskHook`]
    /// - [`OnHookAttach`]
    pub async fn attach_persistent<T, E>(&self, ctx: &TaskContext, hook: Arc<T>)
    where
        E: TaskHookEvent,
        T: TaskHook<E> + PersistenceObject,
    {
        let hook_id = hook.type_id();
        let erased_hook: Arc<dyn ErasedTaskHook> =
            Arc::new(ErasedTaskHookWrapper::<E>(hook.clone(), PhantomData));

        self.0.entry(E::PERSISTENCE_ID).or_default().0.insert(
            hook_id,
            UnknownErasedTaskHook::Persistent {
                id: T::PERSISTENCE_ID,
                hook: erased_hook,
            },
        );
        self.emit::<OnHookAttach<E>>(ctx, &(hook as Arc<dyn TaskHook<E>>))
            .await;
    }

    /// Gets the [`TaskHook`] in the container as immutable,
    /// based on the [`TaskHook`] type and [`TaskHookEvent`] type
    ///
    /// # Returns
    /// The corresponding [`TaskHook`] instance based on the 2 generics supplied
    ///
    /// # See Also
    /// - [`TaskHookContainer`]
    /// - [`TaskHookEvent`]
    /// - [`TaskHook`]
    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        let interested_event_container = self.0.get(E::PERSISTENCE_ID)?;

        let entry = interested_event_container.0.get(&TypeId::of::<T>())?;
        let entry = entry.value().extract();

        entry.clone().as_arc_any().downcast::<T>().ok()
    }

    /// Detaches a [`TaskHook`] from the container, when detached, the [`TaskHook`]
    /// is alerted via [`OnHookDetach`], in which, it knows the [`TaskHookEvent`] it
    /// is detached from
    ///
    /// # See Also
    /// - [`TaskHookContainer`]
    /// - [`TaskHookEvent`]
    /// - [`TaskHook`]
    /// - [`OnHookDetach`]
    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self, ctx: &TaskContext) {
        let Some(event_category) = self.0.get_mut(E::PERSISTENCE_ID) else {
            return;
        };

        let Some((_, hook)) = event_category.0.remove(&TypeId::of::<T>()) else {
            return;
        };

        let erased = hook.extract().clone();

        let typed: Arc<T> = erased
            .as_arc_any()
            .downcast()
            .expect("TaskHook is not of type T (some other is masquerading the 'T' TaskHook)");

        self.emit::<OnHookDetach<E>>(ctx, &(typed as Arc<dyn TaskHook<E>>))
            .await;
    }

    pub async fn emit<E: TaskHookEvent>(&self, ctx: &TaskContext, payload: &E::Payload) {
        if let Some(entry) = self.0.get(E::PERSISTENCE_ID) {
            for hook in entry.value().0.iter() {
                hook.value()
                    .extract()
                    .on_emit(ctx, payload as &(dyn Any + Send + Sync))
                    .await;
            }
        }
    }
}
