use std::any::Any;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use dashmap::DashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;
use crate::task::TaskError;

#[allow(unused_imports)]
use std::collections::HashMap;
use crate::errors::ChronographerErrors;

/// [`ObserverFieldListener`] is the mechanism that drives the reactivity of [`ObserverField`],
/// where it reacts to any changes made to the value
#[async_trait]
pub trait ObserverFieldListener<T: Send + Sync + 'static>: Send + Sync {
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
/// used in metadata to ensure listeners react to changes made to the field
pub struct ObserverField<T: Send + Sync + 'static> {
    value: ArcSwap<T>,
    listeners: Arc<DashMap<Uuid, Arc<dyn ObserverFieldListener<T>>>>,
}

impl<T: Send + Sync + 'static> ObserverField<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: ArcSwap::from_pointee(initial),
            listeners: Arc::new(DashMap::new()),
        }
    }

    pub fn subscribe(&self, listener: impl ObserverFieldListener<T> + 'static) -> Uuid {
        let id = Uuid::new_v4();
        self.listeners.insert(id, Arc::new(listener));
        id
    }

    pub fn unsubscribe(&self, id: &Uuid) {
        self.listeners.remove(id);
    }

    pub fn update(&self, value: T) {
        self.value.store(Arc::new(value));
        self.tap();
    }

    pub fn tap(&self) {
        for listener in self.listeners.iter() {
            let cloned_listener = listener.value().clone();
            let clone_value = self.value.load().clone();
            tokio::spawn(async move {
                cloned_listener.listen(clone_value).await;
            });
        }
    }

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

impl<T: Send + Sync + Clone> Clone for ObserverField<T> {
    fn clone(&self) -> Self {
        ObserverField {
            value: ArcSwap::from(self.value.load_full()),
            listeners: self.listeners.clone(),
        }
    }
}

/// [`TaskMetadata`] is a container hosting all metadata-related information which lives inside a
/// [`Task`]. Depending on the implementation of this trait, the metadata can have static fields or
/// dynamic fields or both, all of them being reactive to any change made. One can also supply the
/// generic ``V`` with a value of their choice for type safety
///
/// When one implements the trait, one can also simply add their own public fields (idiomatically
/// they should be ``ObserverField<T>`` fields) to the mix to provide compile-time safety
///
/// # Required Method(s)
/// Primarily [`TaskMetadata`] requires three of them, those being:
/// - [`TaskMetadata::field`] Accesses a dynamic field and returns an ``Optional<ObserverField<V>>``
/// where its ``None`` if not found and ``Some(ObserverField<V>)`` if found
///
/// - [`TaskMetadata::add_field`] Adds a new dynamic field to the metadata with its own key string
/// and an initial value. If a field already exists, this should ideally return an error as it doesn't
/// behave as much as a typical [`HashMap`] where a new field is added while it exists, it gets modified
///
/// - [`TaskMetadata::remove_field`] Removes the dynamic field based on a ``key`` string,
/// if the dynamic field doesn't exist based on the key, then it does nothing
///
/// - [`TaskMetadata::exists`] Checks whenever a field based on a ``key`` exists and returns
/// a boolean value indicating so, true for if it exists and false otherwise
///
/// # Trait Implementation(s)
/// In ChronoGrapher, there is only one implementation out there of the trait [`TaskMetadata`],
/// and that is [`DynamicTaskMetadata`] which acts as a wrapper around ``DashMap`` and allows for only
/// dynamic fields
///
/// # See Also
/// - [`Task`]
pub trait TaskMetadata<V: Send + Sync + 'static = &'static (dyn Any + Send + Sync)>: Send + Sync
where
    ObserverField<V>: Clone
{
    fn field(&self, key: &str) -> Option<ObserverField<V>>;
    fn add_field(&self, key: &str, value: V) -> Result<(), TaskError>;
    fn remove_field(&self, key: &str);
    fn exists(&self, key: &str) -> bool;
}

#[derive(Clone, Debug)]
pub struct DynamicTaskMetadata<V: Send + Sync + 'static>(DashMap<String, ObserverField<V>>)
where
    ObserverField<V>: Clone;

impl<V: Send + Sync + 'static> Default for DynamicTaskMetadata<V>
where
    ObserverField<V>: Clone
{
    fn default() -> Self {
        Self(DashMap::new())
    }
}

impl<V: Send + Sync + 'static> DynamicTaskMetadata<V>
where
    ObserverField<V>: Clone
{
    pub fn new() -> Self {
        Self(DashMap::new())
    }
}

impl<V: Send + Sync + 'static> TaskMetadata<V> for DynamicTaskMetadata<V>
where
    ObserverField<V>: Clone
{
    fn field(&self, key: &str) -> Option<ObserverField<V>> {
        self.0.get(key).map(|x| x.value().clone())
    }

    fn add_field(&self, key: &str, value: V) -> Result<(), TaskError> {
        if self.0.contains_key(key) {
            return Err(Arc::new(ChronographerErrors::DynamicFieldAlreadyExists(key.to_string())))
        }
        self.0.insert(key.to_string(), ObserverField::new(value));
        Ok(())
    }

    fn remove_field(&self, key: &str) {
        self.0.remove(key);
    }

    fn exists(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}