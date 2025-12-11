use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::TaskError;
#[allow(unused_imports)]
use crate::task::frames::*;
use crate::{define_event, define_generic_event};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

pub mod events {
    pub use crate::task::OnTaskEnd;
    pub use crate::task::OnTaskStart;
    pub use crate::task::frames::OnChildEnd;
    pub use crate::task::frames::OnChildStart;
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
    pub use crate::task::hooks::OnHookAttach;
    pub use crate::task::hooks::OnHookDetach;
    pub use crate::task::hooks::TaskHookEvent;
} // skipcq: RS-D1001

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
/// - [`OnChildStart`]
/// - [`OnChildEnd`]
/// - [`OnTaskFrameSelection`]
/// - [`OnFallbackEvent`]
/// - [`OnDependencyValidation`]
pub trait TaskHookEvent:
    Send + Sync + Default + 'static + Serialize + for<'de> Deserialize<'de>
{
    type Payload: Send + Sync;
    const PERSISTENCE_ID: &'static str;
}

pub enum NonEmissible {}

impl TaskHookEvent for () {
    type Payload = NonEmissible;
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
/// ```rust
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
    /// # See Also
    OnTaskStart, ()
);

define_event!(
    /// # See Also
    OnTaskEnd, Option<TaskError>
);

define_generic_event!(OnHookAttach);
define_generic_event!(OnHookDetach);

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
    pub(crate) DashMap<&'static str, DashMap<TypeId, Arc<dyn ErasedTaskHook>>>,
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
    pub async fn attach<E: TaskHookEvent>(&self, ctx: &TaskContext, hook: Arc<dyn TaskHook<E>>) {
        let hook_id = hook.type_id();
        let hook: Arc<dyn ErasedTaskHook> = Arc::new(ErasedTaskHookWrapper::<E>(hook, PhantomData));

        self.0
            .entry(E::PERSISTENCE_ID)
            .or_default()
            .insert(hook_id, hook);
        self.emit::<OnHookAttach<E>>(ctx, &E::default()).await;
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

        let entry = interested_event_container.get(&TypeId::of::<T>())?;

        let entry = entry.value();

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
        self.0
            .get_mut(E::PERSISTENCE_ID)
            .map(|x| x.remove(&TypeId::of::<T>()));
        self.emit::<OnHookDetach<E>>(ctx, &E::default()).await;
    }

    pub async fn emit<E: TaskHookEvent>(&self, ctx: &TaskContext, payload: &E::Payload) {
        if let Some(entry) = self.0.get(E::PERSISTENCE_ID) {
            for hook in entry.value().iter() {
                hook.value()
                    .on_emit(ctx, payload as &(dyn Any + Send + Sync))
                    .await;
            }
        }
    }
}
