use arc_swap::ArcSwap;
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[allow(unused_imports)]
use std::collections::HashMap;

/// [`MetadataEventListener`] is a function tailored to listening to [`TaskMetadata`] events, as
/// it accepts metadata and a payload as arguments but returns nothing, only really being useful for
/// just listening to relevant [`MetadataEvent`] fires. Functions and closures automatically implement
/// this trait, but due to their nature they cannot persist, as a result, it is advised to create
/// your own struct and implement this trait
///
/// # Required Method(s)
/// When implementing the [`MetadataEventListener`] trait, one has to supply an implementation for
/// the method [`MetadataEventListener::execute`] which accepts the metadata and a payload (which contains
/// additional parameters depending on the event)
///
/// # See Also
/// - [`MetadataEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
#[async_trait]
pub trait MetadataEventListener<P: Send + Sync>: Send + Sync {
    async fn execute(&self, metadata: Arc<TaskMetadata>, payload: Arc<P>);
}

#[async_trait]
impl<P, F, Fut> MetadataEventListener<P> for F
where
    P: Send + Sync + 'static,
    F: Fn(Arc<TaskMetadata>, Arc<P>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn execute(&self, metadata: Arc<TaskMetadata>, payload: Arc<P>) {
        self(metadata, payload).await;
    }
}

#[async_trait]
impl<P: Send + Sync + 'static, E: MetadataEventListener<P> + ?Sized> MetadataEventListener<P>
    for Arc<E>
{
    async fn execute(&self, metadata: Arc<TaskMetadata>, payload: Arc<P>) {
        self.as_ref().execute(metadata, payload).await;
    }
}

/// [`MetadataEvent`] defines an event tailored to [`TaskMetadata`]. This is the main system used for
/// listening to various events via [`MetadataEventListener`] happening in [`TaskMetadata`], as
/// of now there are 2 of these, those being when an insert happens in [`TaskMetadata`] and when
/// a remove happens in [`TaskMetadata`]
///
/// [`MetadataEvent`] **CANNOT** be emitted outside a [`TaskMetadata`], the emission is handled
/// automatically by [`TaskMetadata`].
///
/// # Construction(s)
/// When constructing a [`MetadataEvent`] the one can construct it via [`MetadataEvent::new`] which returns
/// an ``Arc<MetadataEvent<P>>`` where ``P`` is a payload or via [`MetadataEvent::default`] from the [`Default`]
///
/// # Trait Implementation(s)
/// [`MetadataEvent`] implements the [`Default`] trait only
///
/// # See Also
/// - [`MetadataEventListener`]
/// - [`TaskMetadata`]
pub struct MetadataEvent<P> {
    listeners: DashMap<Uuid, Arc<dyn MetadataEventListener<P>>>,
}

impl<P: Send + Sync + 'static> MetadataEvent<P> {
    /// Creates / Constructs a [`MetadataEvent`], containing no [`MetadataEventListener`] and wrapped in an ``Arc``,
    /// which developers can use throughout their codebase, this is exactly the same as doing it
    /// with [`MetadataEvent::default`] in the form of:
    /// ```ignore
    /// Arc::new(MetadataEvent::default())
    /// ```
    ///
    /// # Returns
    /// The constructed [`MetadataEvent`] wrapped in an ``Arc<MetadataEvent<P>>``
    /// where ``P`` is the payload
    ///
    /// # See Also
    /// - [`MetadataEvent`]
    /// - [`MetadataEvent::default`]
    /// - [`MetadataEventListener`]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            listeners: DashMap::new(),
        })
    }

    /// Subscribes a [`MetadataEventListener`] to the [`MetadataEvent`], returning an
    /// identifier for that listener / subscriber
    ///
    /// # Arguments
    /// This method accepts only one argument, that being an implementation of [`MetadataEventListener`]
    /// with a specified payload
    ///
    /// # Returns
    /// An identifier as a UUID to later unsubscribe the [`MetadataEventListener`]
    /// from that event via [`MetadataEvent::unsubscribe`]
    ///
    /// # See Also
    /// - [`MetadataEventListener`]
    /// - [`MetadataEvent`]
    /// - [`MetadataEvent::unsubscribe`]
    pub async fn subscribe(&self, func: impl MetadataEventListener<P> + 'static) -> Uuid {
        let id = Uuid::new_v4();
        self.listeners.insert(id, Arc::new(func));
        id
    }

    /// Unsubscribes a [`MetadataEventListener`] from the [`MetadataEvent`], based on an identifier (UUID),
    /// this identifier is returned when calling [`MetadataEvent::subscribe`] with an [`MetadataEventListener`]
    ///
    /// # Arguments
    /// This method accepts only one argument, that being the UUID corresponding to the
    /// [`MetadataEventListener`], if the UUID isn't associated with a [`MetadataEventListener`] then nothing
    /// happens
    ///
    /// # See Also
    /// - [`MetadataEventListener`]
    /// - [`MetadataEvent`]
    /// - [`MetadataEvent::subscribe`]
    pub async fn unsubscribe(&self, id: &Uuid) {
        self.listeners.remove(id);
    }

    fn emit(&self, metadata: Arc<TaskMetadata>, payload: P) {
        let payload_arc = Arc::new(payload);
        for listener in self.listeners.iter() {
            let cloned_listener = listener.value().clone();
            let cloned_payload = payload_arc.clone();
            let cloned_metadata = metadata.clone();
            tokio::spawn(async move {
                cloned_listener
                    .execute(cloned_metadata, cloned_payload)
                    .await;
            });
        }
    }
}

/// [`ObserverFieldListener`] is the mechanism that drives the listening of reactivity on the
/// [`ObserverField`], where it listens to any changes made to the value. This system is used
/// closely on [`TaskMetadata`] for both static and dynamic fields
///
/// # Required Method(s)
/// When implementing the [`ObserverFieldListener`], one has to implement the [`ObserverFieldListener::listen`]
/// method which is used for executing logic when a value ``T`` changes. It accepts the value as an
/// ``Arc<T>`` (keep in mind it is not the [`ObserverField`] but rather the inner value of the [`ObserverField`])
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`TaskMetadata`] struct
///
/// # See Also
/// - [`ObserverField`]
/// - [`TaskMetadata`]
#[async_trait]
pub trait ObserverFieldListener<T: Send + Sync + ?Sized>: Send + Sync {
    async fn listen(&self, value: Arc<T>);
}

#[async_trait]
impl<T, F, Fut> ObserverFieldListener<T> for F
where
    T: Send + Sync + 'static,
    F: (for<'a> Fn(Arc<T>) -> Fut) + Send + Sync,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn listen(&self, value: Arc<T>) {
        self(value).await;
    }
}

/*
    I am aware that I almost do the same in TaskEvent, however they differ in the fact
    that mutating fields can be done by anyone, whereas event emotion is only done by the
    scheduler or task frame
*/

/// [`ObserverField`] is a reactive container around a field, it is commonly
/// used in [`TaskMetadata`] to ensure [`ObserverFieldListener`] react to field changes.
/// It is a struct which wraps an ``ArcSwap<T>`` where ``T`` is the value to be wrapped
/// and hosts multiple listeners to listen to that field
///
/// # Constructor(s)
/// Constructing a [`ObserverField`], one can call [`ObserverField::new`] and supply it with an
/// initial value, which will create the [`ObserverField`] wrapper around the value
///
/// # See Also
/// - [`ObserverFieldListener`]
/// - [`TaskMetadata`]
pub struct ObserverField<T: Send + Sync + 'static> {
    value: ArcSwap<T>,
    listeners: Arc<DashMap<Uuid, Arc<dyn ObserverFieldListener<T>>>>,
}

impl<T: Send + Sync + 'static> ObserverField<T> {
    /// Constructs / Creates a new [`ObserverField`] instance and returns it for
    /// the developers to use it throughout the codebase
    ///
    /// # Argument(s)
    /// It accepts only one argument, that being an initial value of type ``T``
    ///
    /// # Returns
    /// The constructed instance of [`ObserverField`]
    pub fn new(initial: T) -> Self {
        Self {
            value: ArcSwap::from_pointee(initial),
            listeners: Arc::new(DashMap::new()),
        }
    }

    /// Subscribes via a listener to the current [`ObserverField`] instance,
    /// returning a UUID v4 that points to the listener (if one wishes to unsubscribe)
    ///
    /// # Argument(s)
    /// It accepts only one argument, that being an implementation of [`ObserverFieldListener`],
    /// which is the logic that will be executed when a value changes
    ///
    /// # Returns
    /// The UUID pointing to the [`ObserverFieldListener`] entry, which can be used to later
    /// unsubscribe from the changes that [`ObserverField`] announces via [`ObserverField::unsubscribe`]
    ///
    /// # See Also
    /// - [`ObserverFieldListener`]
    /// - [`ObserverField`]
    /// - [`ObserverField::unsubscribe`]
    pub fn subscribe(&self, listener: impl ObserverFieldListener<T> + 'static) -> Uuid {
        let id = Uuid::new_v4();
        self.listeners.insert(id, Arc::new(listener));
        id
    }

    /// Unsubscribes a listener from the current [`ObserverField`] instance, via
    /// a UUID reference, if it doesn't exist, then it does nothing
    ///
    /// # Argument(s)
    /// It accepts only one argument, that being a UUID reference pointing to a potential
    /// [`ObserverFieldListener`], this UUID is acquired via [`ObserverField::subscribe`]
    ///
    /// # See Also
    /// - [`ObserverFieldListener`]
    /// - [`ObserverField`]
    /// - [`ObserverField::subscribe`]
    pub fn unsubscribe(&self, id: &Uuid) {
        self.listeners.remove(id);
    }

    /// Updates the [`ObserverField`] with a new value, alerting all contained
    /// [`ObserverFieldListener`], under the hood it uses [`ObserverField::tap`] for
    /// notifying (which in its own does nothing but notify)
    ///
    /// # Argument(s)
    /// It accepts only one argument, that being the new value ``T``
    /// to update this [`ObserverField`] with
    ///
    /// # See Also
    /// - [`ObserverFieldListener`]
    /// - [`ObserverField`]
    /// - [`ObserverField::tap`]
    pub fn update(&self, value: T) {
        self.value.store(Arc::new(value));
        self.tap();
    }

    /// Notifies all [`ObserverFieldListener`] contained inside [`ObserverField`].
    /// This does not change the value to anything new, it is only for manual notification,
    /// even if a change hasn't happened to the value. This is used by [`ObserverField::update`]
    /// under the hood
    ///
    /// # See Also
    /// - [`ObserverFieldListener`]
    /// - [`ObserverField`]
    /// - [`ObserverField::update`]
    pub fn tap(&self) {
        for listener in self.listeners.iter() {
            let cloned_listener = listener.value().clone();
            let clone_value = self.value.load().clone();
            tokio::spawn(async move {
                cloned_listener.listen(clone_value).await;
            });
        }
    }

    /// Gets the inner value of [`ObserverField`] as an ``Arc<T>`` where ``T`` is
    /// the type of the inner value and returns it
    ///
    /// # Returns
    /// The inner value acquired from the [`ObserverField`]
    ///
    /// # See Also
    /// - [`ObserverField`]
    pub fn get(&self) -> Arc<T> {
        self.value.load().clone()
    }
}

impl<T: Send + Sync + Display + 'static> Display for ObserverField<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Send + Sync + Debug + 'static> Debug for ObserverField<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ObserverField").field(&self.value).finish()
    }
}

impl<T: Send + Sync + PartialEq + 'static> PartialEq<Self> for ObserverField<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(&other.get())
    }
}

impl<T: Send + Sync + Eq + 'static> Eq for ObserverField<T> {}

impl<T: Send + Sync + PartialOrd + 'static> PartialOrd for ObserverField<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: Send + Sync + Ord + 'static> Ord for ObserverField<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: Send + Sync> Clone for ObserverField<T> {
    fn clone(&self) -> Self {
        ObserverField {
            value: ArcSwap::from(self.value.load_full()),
            listeners: self.listeners.clone(),
        }
    }
}

/// The type alias for ``on_insert`` event
pub type OnMetadataInsertEvent =
    Arc<MetadataEvent<(String, ObserverField<Box<dyn Any + Send + Sync + 'static>>)>>;

/// The type alias for ``on_remove`` event
pub type OnMetadataRemoveEvent = Arc<MetadataEvent<String>>;

/// [`TaskMetadata`] is a reactive container, which hosts keys as strings
/// and values as [`ObserverField`] with any value inside. It acts more as a glorified
/// wrapper around ``DashMap``
///
/// # Constructor(s)
/// You can either construct it via [`TaskMetadata::default`] via [`Default`] or you can
/// construct it via [`TaskMetadata::new`], both do the same and that is initializing a new
/// empty task metadata
///
/// # Trait Implementation(s)
/// [`TaskMetadata`] as previously mentioned, implements the [`Default`] trait, but in addition
/// it also implements the [`Debug`] trait
pub struct TaskMetadata {
    fields: DashMap<String, ObserverField<Box<dyn Any + Send + Sync + 'static>>>,

    /// Event fired when an insertion happens
    pub on_insert: OnMetadataInsertEvent,

    /// Event fired when a removal happens
    pub on_remove: OnMetadataRemoveEvent,
}

impl Debug for TaskMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskMetadata")
            .field("fields", &self.fields)
            .finish()
    }
}

impl Default for TaskMetadata {
    fn default() -> Self {
        Self {
            fields: DashMap::new(),
            on_insert: MetadataEvent::new(),
            on_remove: MetadataEvent::new(),
        }
    }
}

impl TaskMetadata {
    /// Constructs / Creates a new empty [`TaskMetadata`] instance for developers
    /// to use throughout their code
    ///
    /// # Returns
    /// The constructed [`TaskMetadata`] instance to be used
    ///
    /// # See Also
    /// - [`TaskMetadata`]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            fields: DashMap::new(),
            on_insert: MetadataEvent::new(),
            on_remove: MetadataEvent::new(),
        })
    }

    /// Gets a potential [`ObserverField`] based on a key and returns it, if it doesn't exist,
    /// one can use [`TaskMetadata::insert`] to append a new entry and then acquire it, one
    /// can also check if the entry exists via [`TaskMetadata::exists`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being a key as a ``&str`` which points
    /// to the [`ObserverField`] you want to get
    ///
    /// # Returns
    /// The [`ObserverField`] wrapped as an option, if it has been found then it returns
    /// ``Some`` otherwise ``None`` if it hasn't been found with that key
    ///
    /// # See Also
    /// - [`ObserverField`]
    /// - [`TaskMetadata`]
    /// - [`TaskMetadata::insert`]
    /// - [`TaskMetadata::exists`]
    pub fn get(self: Arc<Self>, key: &str) -> Option<ObserverField<Box<dyn Any + Send + Sync>>> {
        Some(self.fields.get(key)?.value().clone())
    }

    /// Inserts a new entry of a key and a value, then returns a boolean. This is the opposite
    /// of [`TaskMetadata::remove`], one can also check if the key exists via [`TaskMetadata::exists`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being a key as an owned``String`` and
    /// an initial value which is an implementation of ``Any + Send + Sync``
    ///
    /// # Returns
    /// A boolean indicating if there was any previous value that had the same key (idiomatically
    /// one shouldn't replace values with [`TaskMetadata::insert`] but rather use [`ObserverField`]
    /// for updating them via getting them [`TaskMetadata::get`])
    ///
    /// # See Also
    /// - [`ObserverField`]
    /// - [`TaskMetadata`]
    /// - [`TaskMetadata::get`]
    /// - [`TaskMetadata::remove`]
    /// - [`TaskMetadata::exists`]
    pub fn insert(self: Arc<Self>, key: String, value: impl Any + Send + Sync) -> bool {
        let value: Box<dyn Any + Send + Sync> = Box::new(value);
        let field = ObserverField::new(value);
        let cloned_key = key.clone();
        let result = self.fields.insert(key, field.clone()).is_none();
        self.on_insert.emit(self.clone(), (cloned_key, field));
        result
    }

    /// Removes an entry via a key, if the entry doesn't exist based on the key, then
    /// nothing happens.  This is the opposite of [`TaskMetadata::remove`], one can also check if
    /// the key exists via [`TaskMetadata::exists`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being a key as a``&str`` which points to
    /// the [`ObserverField`] to be removed along with the key (the entry)
    ///
    /// # See Also
    /// - [`ObserverField`]
    /// - [`TaskMetadata`]
    /// - [`TaskMetadata::insert`]
    /// - [`TaskMetadata::exists`]
    pub fn remove(self: Arc<Self>, key: String) {
        self.fields.remove(&key);
        self.on_remove.emit(self.clone(), key.clone());
    }

    /// Checks if an entry exists via a key, this is the same as just using
    /// [`TaskMetadata::get`] and checking if the value is some
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being a key as a``&str`` which points to
    /// the [`ObserverField`] to be removed along with the key (the entry)
    ///
    /// # See Also
    /// - [`ObserverField`]
    /// - [`TaskMetadata`]
    /// - [`TaskMetadata::get`]
    pub fn exists(self: Arc<Self>, key: &str) -> bool {
        self.fields.contains_key(key)
    }
}
