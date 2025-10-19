use crate::persistent_object::PersistentObject;
use crate::retrieve_registers::RetrieveRegistries;
use crate::serialized_component::SerializedComponent;
use crate::task::{TaskContext, TaskError};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// The ``on_start`` event type alias
pub type TaskStartEvent = ArcTaskEvent<()>;

/// The ``on_end`` event type alias
pub type TaskEndEvent = ArcTaskEvent<Option<TaskError>>;

/// A convenient type alias for wrapping a task event in an ``Arc<T>``
pub type ArcTaskEvent<P> = Arc<TaskEvent<P>>;

/// [`TaskEventListener`] is a function tailored to listening to [`Task`] and [`TaskFrame`] events, as
/// it accepts restricted [`TaskContext`] and a payload as arguments but returns nothing, only really being useful for
/// just listening to relevant [`TaskEvent`] fires. Functions and closures automatically implement
/// this trait, but due to their nature they cannot persist, as a result, it is advised to create
/// your own struct and implement this trait
///
/// # Required Method(s)
/// When implementing the [`TaskEventListener`] trait, one has to supply an implementation for
/// the method [`TaskEventListener::execute`] which accepts a restricted [`TaskContext`] and a payload (which contains
/// additional parameters depending on the event)
///
/// # See Also
/// - [`TaskEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
/// - [`TaskContext`]
#[async_trait]
pub trait TaskEventListener<P: Send + Sync>: Send + Sync {
    async fn execute(&self, ctx: Arc<TaskContext<true>>, payload: Arc<P>);
}

#[async_trait]
impl<P, F, Fut> TaskEventListener<P> for F
where
    P: Send + Sync + 'static,
    F: Fn(Arc<TaskContext<true>>, Arc<P>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn execute(&self, ctx: Arc<TaskContext<true>>, payload: Arc<P>) {
        self(ctx, payload).await;
    }
}

#[async_trait]
impl<P: Send + Sync + 'static, E: TaskEventListener<P>> TaskEventListener<P> for Arc<E> {
    async fn execute(&self, ctx: Arc<TaskContext<true>>, payload: Arc<P>) {
        self.as_ref().execute(ctx, payload).await;
    }
}

/// [`TaskEvent`] defines an event which may (or may not, depending on how the frame implementation
/// handles this task event) execute. This is the main system used for listening to various events,
/// there are 2 types of events at play, which one can listen to:
///
/// - **Lifecycle Task Events** These live inside [`Task`], and specifically are 2 events, the former
///   is ``on_start`` which executes always when the task is about to start. While the latter is
///   ``on_end`` which is always executed before the error handler and after the task execution finishes,
///   these events tackle the lifecycle of a task as such the reason why they are named like so
///
/// - **Local Task Frame Events** These are local to the task frame, different task frames may have none, one
///   or multiple of these event types. They are emitted by the task frame logic and give more extensibility
///   to what outside parties can listen to (for example, on the fallback task frame, one can listen to
///   when the fallback is about to execute)
///
/// [`TaskEvent`] **CANNOT** be emitted by itself, it needs an emitter which is only handed to the
/// scheduler, overlapping policies and the task frame. Outside parties can listen to the event at any
/// time they would like
///
/// # Construction(s)
/// When constructing a [`TaskEvent`] the one can construct it via [`TaskEvent::new`] which returns
/// an ``Arc<TaskEvent<P>>`` where ``P`` is a payload or via [`TaskEvent::default`] from the [`Default`]
///
/// # Trait Implementation(s)
/// [`TaskEvent`] implements the [`Default`] trait only
///
/// # See Also
/// - [`TaskEventListener`]
/// - [`Task`]
pub struct TaskEvent<P> {
    listeners: DashMap<Uuid, Arc<dyn TaskEventListener<P>>>,
}

impl<P: Send + Sync + 'static> Default for TaskEvent<P> {
    fn default() -> Self {
        Self {
            listeners: DashMap::new(),
        }
    }
}

impl<P: Send + Sync + 'static> TaskEvent<P> {
    /// Creates / Constructs a [`TaskEvent`], containing no [`TaskEventListener`] and wrapped in an ``Arc``,
    /// which developers can use throughout their codebase, this is exactly the same as doing it
    /// with [`TaskEvent::default`] in the form of:
    /// ```ignore
    /// Arc::new(TaskEvent::default())
    /// ```
    ///
    /// # Returns
    /// The constructed [`TaskEvent`] wrapped in an ``Arc<TaskEvent<P>>``
    /// where ``P`` is the payload
    ///
    /// # See Also
    /// - [`TaskEvent`]
    /// - [`TaskEvent::default`]
    /// - [`TaskEventListener`]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            listeners: DashMap::new(),
        })
    }

    /// Subscribes a [`TaskEventListener`] to the [`TaskEvent`], returning an identifier for that
    /// listener / subscriber
    ///
    /// # Arguments
    /// This method accepts only one argument, that being an implementation of [`TaskEventListener`]
    /// with a specified payload
    ///
    /// # Returns
    /// An identifier as a UUID to later unsubscribe the [`TaskEventListener`]
    /// from that event via [`TaskEvent::unsubscribe`]
    ///
    /// # See Also
    /// - [`TaskEventListener`]
    /// - [`TaskEvent`]
    /// - [`TaskEvent::unsubscribe`]
    pub async fn subscribe(&self, func: impl TaskEventListener<P> + 'static) -> Uuid {
        let id = Uuid::new_v4();
        self.listeners.insert(id, Arc::new(func));
        id
    }

    /// Unsubscribes a [`TaskEventListener`] from the [`TaskEvent`], based on an identifier (UUID),
    /// this identifier is returned when calling [`TaskEvent::subscribe`] with an [`TaskEventListener`]
    ///
    /// # Arguments
    /// This method accepts only one argument, that being the UUID corresponding to the
    /// [`TaskEventListener`], if the UUID isn't associated with a [`TaskEventListener`] then nothing
    /// happens
    ///
    /// # See Also
    /// - [`TaskEventListener`]
    /// - [`TaskEvent`]
    /// - [`TaskEvent::subscribe`]
    pub async fn unsubscribe(&self, id: &Uuid) {
        self.listeners.remove(id);
    }
}

#[async_trait]
impl<P: Send + Sync + 'static> PersistentObject for TaskEvent<P> {
    fn persistence_id() -> &'static str {
        "chronographer_core$TaskEvent"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let mut serialized = Vec::with_capacity(self.listeners.len());
        for entry in self.listeners.iter() {
            let id = entry.key();
            let listener = entry.value();
            let serialized_listener = PersistenceUtils::serialize_potential_field(listener).await?;
            let serialized_id = PersistenceUtils::serialize_field(id.to_u128_le())?;
            serialized.push(json!({
                "id": serialized_id,
                "listener": serialized_listener
            }));
        }
        Ok(SerializedComponent::new::<Self>(json!({
            "listeners": PersistenceUtils::serialize_field(serialized)?
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;
        let mut partially_serialized_listeners =
            PersistenceUtils::deserialize_partially_field::<Self>(
                &mut repr,
                "listeners",
                "Cannot deserialize the listeners",
            )?;

        let serialized_listeners = partially_serialized_listeners.as_array_mut().ok_or(
            PersistenceUtils::create_retrieval_error::<Self>(
                &repr,
                "Cannot deserialize the listeners",
            ),
        )?;

        let listeners = DashMap::new();

        for entry in serialized_listeners.iter_mut() {
            let mut entry = entry.as_object_mut().ok_or_else(|| {
                PersistenceUtils::create_retrieval_error::<Self>(
                    &repr,
                    "Failed to deserialize a listener",
                )
            })?;

            let id = Uuid::from_u128_le(PersistenceUtils::deserialize_atomic::<u128>(
                &mut entry,
                "id",
                "Failed to deserialize the ID of a listener",
            )?);

            let listener = PersistenceUtils::deserialize_dyn(
                &mut entry,
                "listener",
                RetrieveRegistries::retrieve_task_event_listener,
                "Failed to deserialize the listener function",
            )
            .await?;

            listeners.insert(id, listener);
        }

        Ok(TaskEvent { listeners })
    }
}

/// [`TaskEventEmitter`] is a sealed mechanism to allow the use of emitting [`TaskEvent`] which
/// alerts all [`TaskEventListener`], by itself it doesn't hot any data, but it unlocks the use
/// of [`TaskEventEmitter::emit`]. The reason for this is to prevent any emissions from outside parties
/// on [`TaskEvent`]
///
/// # Constructor(s)
/// There are no constructors for public use, it cannot be constructed via a constructor method
/// nor via rust's struct initialization from the public, internally ChronoGrapher constructs it
///
/// # See Also
/// - [`TaskEvent`]
/// - [`TaskEventListener`]
pub struct TaskEventEmitter {
    pub(crate) _private: (),
}

impl TaskEventEmitter {
    /// Emits the [`TaskEvent`], notifying all [`TaskEventListener`] from event
    ///
    /// # Argument(s)
    /// The method accepts 3 arguments, the first being ``ctx`` which is a
    /// restricted version of [`TaskContext`], the second is the actual [`TaskEvent`] via ``event``. While the third is the
    /// payload of the [`TaskEvent`] via ``payload`` (it depends on what payload type the ``event`` has)
    ///
    /// # See Also
    /// - [`TaskEvent`]
    /// - [`TaskEventListener`]
    /// - [`TaskEventEmitter`]
    pub async fn emit<P: Send + Sync + Clone + 'static>(
        &self,
        ctx: Arc<TaskContext<true>>,
        event: Arc<TaskEvent<P>>,
        payload: P,
    ) {
        let payload_arc = Arc::new(payload);
        for listener in event.listeners.iter() {
            let cloned_listener = listener.value().clone();
            let cloned_context = ctx.clone();
            let cloned_payload = payload_arc.clone();
            tokio::spawn(async move {
                cloned_listener
                    .execute(cloned_context, cloned_payload)
                    .await;
            });
        }
    }
}
