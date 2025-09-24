use arc_swap::ArcSwap;
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[allow(unused_imports)]
use std::collections::HashMap;

/// [`ObserverFieldListener`] is the mechanism that drives the listening of reactivity on the
/// [`ObserverField`], where it listens to any changes made to the value. This system is used
/// closely on [`TaskMetadata`] for both static and dynamic fields
///
/// # Required Method(s)
/// When implementing the [`ObserverFieldListener`], one has to implement the [`ObserverFieldListener::listen`]
/// method which is used for executing logic when a value ``T`` changes. It accepts the value as an
/// ``Arc<T>`` (keep in mind it is not the [`ObserverField`] but rather the inner value of the [`ObserverField`])
///
/// # See Also
/// - [`ObserverField`]
/// - [`TaskMetadata`]
#[async_trait]
pub trait ObserverFieldListener<T: Send + Sync + 'static + ?Sized>: Send + Sync {
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
/// # See Also
/// - [`ObserverFieldListener`]
/// - [`TaskMetadata`]
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

impl<T: Send + Sync> Clone for ObserverField<T> {
    fn clone(&self) -> Self {
        ObserverField {
            value: ArcSwap::from(self.value.load_full()),
            listeners: self.listeners.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskMetadata(DashMap<&'static str, ObserverField<dyn Any + Send + Sync + 'static>>);

impl Default for TaskMetadata
{
    fn default() -> Self {
        Self(DashMap::new())
    }
}

impl TaskMetadata {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    pub fn get<V: Any + Send + Sync>(&self, key: &str) -> Option<ObserverField<V>> {
        self.0
            .get(&key)
            .and_then(|f| f.value().downcast_ref::<V>())
    }

    pub fn insert<V: Any + Send + Sync>(&self, key: &str, value: V) -> bool {
        let field = ObserverField::new(Box::new(value));
        self.0.insert(key, field).is_none()
    }

    pub fn remove(&self, key: &str) {
        self.0.remove(key);
    }

    pub fn exists(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}