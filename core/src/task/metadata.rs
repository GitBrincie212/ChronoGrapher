use arc_swap::ArcSwap;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use dashmap::DashMap;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU64;
use std::sync::Arc;
use uuid::Uuid;

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

    pub fn update(&mut self, value: T) {
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

impl<T: Send + Sync + Clone + 'static> Clone for ObserverField<T> {
    fn clone(&self) -> Self {
        ObserverField {
            value: ArcSwap::from(self.value.load_full()),
            listeners: self.listeners.clone(),
        }
    }
}

/// [`TaskMetadata`] is a container hosting all metadata-related information in which one can
/// modify and read from it. It contains common internal metadata such as run count and
/// last execution time (handled by the scheduling logic) as well as non-internal metadata,
/// i.e. Debug label and maximum run count (set by the user)
///
/// [`TaskMetadata`] can be handed as a reference. Providing immutability. For mutable fields, consider
/// using [`ObserverField`] to observe the changes of the field and react appropriately. If a field
/// is meant to be internal, it is advised to make it ``&mut`` even if not modifying the struct
///
/// By default, task metadata contains:
/// - **Maximum runs**, the maximum runs this task can run before stopping the rescheduling (by default
///   it is set to run for infinite times), accessed via [`TaskMetadata::max_runs`]
///
/// - **Run Count**, the number of times this task has run throughout its lifetime,
///   accessed via [`TaskMetadata::runs`]
///
/// - **Last Execution**, the point in which the task was last executed at, if the task hasn't
///   executed then it returns none, accessed via [`TaskMetadata::last_execution`]
///
/// - **Debug Label** a label (can be an ID, a name... etc.) for identifying a task, by default, it
///   constructs a UUID per task, can be accessed via [`TaskMetadata::debug_label`]
pub trait TaskMetadata: Send + Sync {
    fn max_runs(&self) -> Option<NonZeroU64>;
    fn last_execution(&self) -> ObserverField<DateTime<Local>>;
    fn debug_label(&self) -> ObserverField<String>;
}

impl<M: TaskMetadata + ?Sized> TaskMetadata for Arc<M> {
    fn max_runs(&self) -> Option<NonZeroU64> {
        self.as_ref().max_runs()
    }
    fn last_execution(&self) -> ObserverField<DateTime<Local>> {
        self.as_ref().last_execution()
    }
    fn debug_label(&self) -> ObserverField<String> {
        self.as_ref().debug_label()
    }
}

/// A default implementation of the [`TaskMetadata`], it contains the bare minimum information
/// to describe a [`TaskMetadata`], this is used for internally tracking state of the task.
/// Unless one wishes to track more state in task metadata, this container is the go-to default
/// option (which is why [`Task`] doesn't require the supply of metadata)
///
/// # See
/// - [`Task`]
/// - [`TaskMetadata`]
/// - [`DefaultTaskMetadataExposed`]
pub struct DefaultTaskMetadata {
    pub(crate) max_runs: Option<NonZeroU64>,
    pub(crate) last_execution: ObserverField<DateTime<Local>>,
    pub(crate) debug_label: ObserverField<String>,
}

impl Default for DefaultTaskMetadata {
    fn default() -> Self {
        DefaultTaskMetadata {
            max_runs: None,
            last_execution: ObserverField::new(Local::now()),
            debug_label: ObserverField::new(Uuid::new_v4().to_string()),
        }
    }
}

impl DefaultTaskMetadata {
    pub fn new() -> Self {
        Self::new_with(Uuid::new_v4().to_string())
    }

    pub fn new_with(debug_label: String) -> Self {
        DefaultTaskMetadata {
            max_runs: None,
            last_execution: ObserverField::new(Local::now()),
            debug_label: ObserverField::new(debug_label),
        }
    }
}

impl TaskMetadata for DefaultTaskMetadata {
    fn max_runs(&self) -> Option<NonZeroU64> {
        self.max_runs
    }
    fn last_execution(&self) -> ObserverField<DateTime<Local>> {
        self.last_execution.clone()
    }
    fn debug_label(&self) -> ObserverField<String> {
        self.debug_label.clone()
    }
}
