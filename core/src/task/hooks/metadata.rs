use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[allow(unused_imports)]
use std::collections::HashMap;

/// [`MetadataEventListener`] is a function tailored to listening to [`TaskMetadata`] events, as
/// it accepts metadata and a key as arguments but returns nothing, only really being useful for
/// just listening to relevant [`MetadataEvent`] fires. Functions and closures automatically implement
/// this trait, but due to their nature they cannot persist, as a result, it is advised to create
/// your own struct and implement this trait
///
/// # Usage Note(s)
/// It is advised to be careful when modifying the metadata, listening to an event and (say ``on_remove`` as
/// an example) and removing an observer field will retrigger this listener. This can potentially lead
/// to infinite recursion on niche cases
///
/// # Required Method(s)
/// When implementing the [`MetadataEventListener`] trait, one has to supply an implementation for
/// the method [`MetadataEventListener::execute`] which accepts the metadata and a key as an ``Arc<str>``
///
/// # See Also
/// - [`MetadataEvent`]
/// - [`Task`]
/// - [`TaskFrame`]
#[async_trait]
pub trait MetadataEventListener: Send + Sync {
    async fn execute(&self, metadata: Arc<TaskMetadata>, key: Arc<str>);
}

#[async_trait]
impl<F, Fut> MetadataEventListener for F
where
    F: Fn(Arc<TaskMetadata>, Arc<str>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn execute(&self, metadata: Arc<TaskMetadata>, payload: Arc<str>) {
        self(metadata, payload).await;
    }
}

#[async_trait]
impl<E> MetadataEventListener for Arc<E>
where
    E: MetadataEventListener + ?Sized,
{
    async fn execute(&self, metadata: Arc<TaskMetadata>, payload: Arc<str>) {
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
pub struct MetadataEvent {
    listeners: DashMap<Uuid, Arc<dyn MetadataEventListener>>,
}

impl MetadataEvent {
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
    pub async fn subscribe(&self, func: impl MetadataEventListener + 'static) -> Uuid {
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

    fn emit(&self, metadata: Arc<TaskMetadata>, key: Arc<str>) {
        for listener in self.listeners.iter() {
            let cloned_listener = listener.value().clone();
            let cloned_key = key.clone();
            let cloned_metadata = metadata.clone();
            tokio::spawn(async move {
                cloned_listener.execute(cloned_metadata, cloned_key).await;
            });
        }
    }
}

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
    fields: DashMap<Arc<str>, ObserverField<Box<dyn Any + Send + Sync + 'static>>>,

    /// Event fired when an insertion happens
    pub on_insert: Arc<MetadataEvent>,

    /// Event fired when a removal happens
    pub on_remove: Arc<MetadataEvent>,
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
        let key: Arc<str> = Arc::from(key);
        let field = ObserverField::new(key.clone(), value);
        let result = self.fields.insert(key.clone(), field.clone()).is_none();
        self.on_insert.emit(self.clone(), key);
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
    pub fn remove(self: Arc<Self>, key: Arc<str>) {
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
