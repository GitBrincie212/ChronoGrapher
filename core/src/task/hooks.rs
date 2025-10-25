pub mod error_handler;
pub mod metadata;

use crate::define_event;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::{Task, TaskError};
use crate::utils::emit_event;
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::any::{Any, TypeId};
use std::fmt::Debug;

#[allow(unused_imports)]
use crate::task::frames::*;

/// [`TaskHookEvent`] is a trait used for describing [`Task`] or [`TaskFrame`] events for [`TaskHook`]
/// instances to hook into. It is meant to be implemented in the form of a marker struct. It contains
/// a payload type for the event (what kind of information does the event include inside) and
/// a Persistence ID
///
/// There are 3 types of [`TaskHookEvent`], These are:
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
/// # Trait Implementation(s)
/// There are multiple implementations for [`TaskHookEvent`] present in ChronoGrapher, almost
/// all [`TaskFrame`] include at least one relevant [`TaskHookEvent`], a list of them are:
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
/// - [`OnChildStart`] - Triggers when a wrapped child [`TaskFrame`] from
/// [`SequentialTaskFrame`] / [`ParallelTaskFrame`] starts
/// - [`OnChildEnd`] - Triggers when a wrapped child [`TaskFrame`] from
/// [`SequentialTaskFrame`] / [`ParallelTaskFrame`] ends
/// - [`OnTaskFrameSelection`] - Triggers when a [`TaskFrame`] is selected from [`SelectTaskFrame`]
/// - [`OnFallbackEvent`] - Triggers when the primary [`TaskFrame`] fails in [`FallbackTaskFrame`]
/// - [`OnDependencyValidation`] - Triggers when a [`TaskDependency`] is validated in [`DependencyTaskFrame`]
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
/// - [`OnChildStart`]
/// - [`OnChildEnd`]
/// - [`OnTaskFrameSelection`]
/// - [`OnFallbackEvent`]
/// - [`OnDependencyValidation`]
pub trait TaskHookEvent: Send + Sync + Default + 'static {
    type Payload: Send + Sync;
    const PERSISTENCE_ID: &'static str;
}

impl<T: TaskHookEvent> PersistentObject for T {
    fn persistence_id() -> &'static str {
        Self::PERSISTENCE_ID
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        Ok(SerializedComponent::new::<Self>(json!({})))
    }

    async fn retrieve(_component: SerializedComponent) -> Result<Self, TaskError> {
        Self
    }
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
pub trait TaskHook<E: TaskHookEvent>: Send + Sync {
    async fn on_event(&self, event: E, payload: &E::Payload);
}

#[async_trait]
pub trait ErasedTaskHook: Send + Sync {
    async fn on_emit(&self, payload: &(dyn Any + Send + Sync));
}

#[async_trait]
impl<E: TaskHookEvent> ErasedTaskHook for dyn TaskHook<E> {
    async fn on_emit(&self, payload: &(dyn Any + Send + Sync)) {
        let payload = payload
            .downcast::<E::Payload>()
            .expect("Invalid payload type passed to TaskHook");
        self.on_event(E::default(), &payload).await;
    }
}

define_event!(
    /// # See Also
    OnTaskStart, Task
);

define_event!(
    /// # See Also
    OnTaskEnd, (&'static Task, Option<TaskError>)
);

/// [`OnHookAttach`] is an implementation of [`TaskHookEvent`] (a system used closely with,
/// [`TaskHook`]). The concrete payload type of [`OnHookAttach`] is ``TaskHookEvent<P>``
///
/// # Constructor(s)
/// When constructing a [`OnHookAttach`], due to the fact this is a marker ``struct``,
/// making it as such zero-sized, one can either use [`OnHookAttach::default`]
/// or via simply pasting the struct name ([`OnHookAttach`])
///
/// # Trait Implementation(s)
/// It is obvious that [`OnHookAttach`] implements the [`TaskHookEvent`], but also many
/// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
/// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
///
/// # Cloning Semantics
/// When cloning / copy a [`OnHookAttach`], it fully creates a
/// new independent version of that instance
///
/// # See Also
/// - [`TaskHook`]
/// - [`TaskHookEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct OnHookAttach<E: TaskHookEvent>;

impl<E: TaskHookEvent> Default for OnHookAttach<E> {
    fn default() -> Self {
        OnHookAttach
    }
}

impl<'a, E: TaskHookEvent> TaskHookEvent for OnHookAttach<E> {
    type Payload = &'a E;
    const PERSISTENCE_ID: &'static str = "chronographer_core#OnHookAttach";
}

/// [`OnHookAttach`] is an implementation of [`TaskHookEvent`] (a system used closely with,
/// [`TaskHook`]). The concrete payload type of [`OnHookAttach`] is ``TaskHookEvent<P>``
///
/// # Constructor(s)
/// When constructing a [`OnHookAttach`], due to the fact this is a marker ``struct``,
/// making it as such zero-sized, one can either use [`OnHookAttach::default`]
/// or via simply pasting the struct name ([`OnHookAttach`])
///
/// # Trait Implementation(s)
/// It is obvious that [`OnHookAttach`] implements the [`TaskHookEvent`], but also many
/// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
/// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
///
/// # Cloning Semantics
/// When cloning / copy a [`OnHookAttach`], it fully creates a
/// new independent version of that instance
///
/// # See Also
/// - [`TaskHook`]
/// - [`TaskHookEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct OnHookDetach<E: TaskHookEvent>;

impl<E: TaskHookEvent> Default for OnHookDetach<E> {
    fn default() -> Self {
        OnHookDetach
    }
}

impl<'a, E: TaskHookEvent> TaskHookEvent for OnHookDetach<E> {
    type Payload = &'a E;
    const PERSISTENCE_ID: &'static str = "chronographer_core#OnHookDetach";
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
pub struct TaskHookContainer(
    pub(crate) DashMap<TypeId, DashMap<TypeId, &'static dyn ErasedTaskHook>>,
);

impl TaskHookContainer {
    /// Attaches a [`TaskHook`] onto the container, when attached, the [`TaskHook`]
    /// is alerted via [`OnHookAttach`], in which, it knows the [`TaskHookEvent`] it is
    /// attached to
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
    pub async fn attach<E: TaskHookEvent>(&self, hook: &impl TaskHook<E>) {
        self.0
            .entry(TypeId::of::<E>())
            .or_default()
            .insert(hook.type_id(), hook);
        emit_event::<OnHookAttach<E>>(self, &()).await;
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
    pub fn get<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<E>())?
            .get(&TypeId::of::<T>())
            .as_ref()
    }

    /// Gets the [`TaskHook`] in the container as mutable,
    /// based on the [`TaskHook`] type and [`TaskHookEvent`] type
    ///
    /// # Returns
    /// The corresponding [`TaskHook`] instance based on the 2 generics supplied
    ///
    /// # See Also
    /// - [`TaskHookContainer`]
    /// - [`TaskHookEvent`]
    /// - [`TaskHook`]
    pub fn get_mut<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<&mut T> {
        self.0
            .get(&TypeId::of::<E>())?
            .get_mut(&TypeId::of::<T>())
            .as_ref()
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
    pub async fn detach<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        self.0
            .get_mut(&TypeId::of::<E>())
            .map(|mut x| x.remove(&TypeId::of::<T>()));
        emit_event::<OnHookDetach<E>>(self, &()).await;
    }
}
