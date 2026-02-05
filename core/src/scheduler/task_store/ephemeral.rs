use crate::errors::ChronographerErrors;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ErasedTask, DynArcError, TriggerNotifier};
use crate::utils::TaskIdentifier;
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, MutexGuard};

struct InternalScheduledItem<C: SchedulerConfig>(SystemTime, C::TaskIdentifier);

impl<C: SchedulerConfig> Eq for InternalScheduledItem<C> {}

impl<C: SchedulerConfig> PartialEq<Self> for InternalScheduledItem<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl<C: SchedulerConfig> PartialOrd<Self> for InternalScheduledItem<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<C: SchedulerConfig> Ord for InternalScheduledItem<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

type EarlyMutexLock<'a, C: SchedulerConfig> = MutexGuard<'a, BinaryHeap<InternalScheduledItem<C>>>;

/// [`EphemeralSchedulerTaskStore`] is an implementation of [`SchedulerTaskStore`]
/// that can operate in-memory and persistence (can be configured with a [`PersistenceBackend`])
///
/// # Usage Note(s)
/// By default [`EphemeralSchedulerTaskStore`] operates in-memory,
/// it doesn't store any information on the disk, while being fast, it makes it brittle
/// to crashes and shutdowns. For enterprise use, it is advised to configure a
/// backend. It is mostly meant to be used for demos or for debugging (where one
/// doesn't care to persist information on disk)
///
/// # Constructor(s)
/// When constructing a new [`EphemeralSchedulerTaskStore`], one can use
/// [`EphemeralSchedulerTaskStore::ephemeral`] for ephemeral-only (in-memory) storage or
/// [`EphemeralSchedulerTaskStore<T>::persistent`] for backend storage with a provided
/// backend
///
/// # Trait Implementation(s)
/// [`EphemeralSchedulerTaskStore`] obviously implements the [`SchedulerTaskStore`]
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use std::time::{Duration, SystemTime};
/// use chronographer_core::clock::VirtualClock;
/// use chronographer_core::scheduler::task_store::EphemeralDefaultTaskStore;
/// use chronographer_core::task::{NoOperationTaskFrame, Task, TaskScheduleInterval};
/// use chronographer_core::scheduler::task_store::SchedulerTaskStore;
///
/// let my_store = EphemeralDefaultTaskStore::new();
/// let my_clock = Arc::new(VirtualClock::from_value(0));
///
/// let primary_task = Task::define(
///     TaskScheduleInterval::from_secs_f64(3.0),
///     NoOperationTaskFrame
/// );
///
/// let secondary_task = Task::define(
///     TaskScheduleInterval::from_secs_f64(1.0),
///     NoOperationTaskFrame
/// );
///
/// let tertiary_task = Task::define(
///     TaskScheduleInterval::from_secs_f64(2.0),
///     NoOperationTaskFrame
/// );
///
/// my_store.store(my_clock.clone(), Arc::new(primary_task)).await;
/// my_store.store(my_clock.clone(), Arc::new(secondary_task)).await;
/// my_store.store(my_clock, Arc::new(tertiary_task)).await;
///
/// my_store.retrieve(); // earliest: primary_task
/// my_store.pop();
/// my_store.retrieve(); // earliest: tertiary_task
/// ```
///
/// # See Also
/// - [`SchedulerTaskStore`]
/// - [`EphemeralSchedulerTaskStore::new`]
pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    earliest_sorted: Arc<Mutex<BinaryHeap<InternalScheduledItem<C>>>>,
    tasks: DashMap<C::TaskIdentifier, Arc<ErasedTask>>,
    sender: tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Result<SystemTime, DynArcError>)>,
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(
            Box<dyn Any + Send + Sync>,
            Result<SystemTime, DynArcError>,
        )>(1024);

        let earliest_sorted = Arc::new(Mutex::new(BinaryHeap::new()));
        let earliest_sorted_clone = earliest_sorted.clone();
        tokio::spawn(async move {
            while let Some((id, time)) = rx.recv().await {
                let id = id.downcast_ref::<C::TaskIdentifier>()
                    .expect("Different type was used on TriggerNotifier, which was meant as for an identifier");
                let mut lock = earliest_sorted_clone.lock().await;
                match time {
                    Ok(time) => lock.push(InternalScheduledItem(time, id.clone())),
                    Err(err) => {
                        eprintln!(
                            "TaskTrigger corresponding to the id {:?} failed to compute a future time with the error {:?}",
                            id, err
                        )
                    }
                }
            }
        });

        Self {
            earliest_sorted,
            tasks: DashMap::new(),
            sender: tx,
        }
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    async fn init(&self) {}

    async fn retrieve(&self) -> Option<(Arc<ErasedTask>, SystemTime, C::TaskIdentifier)> {
        let early_lock: EarlyMutexLock<'_, C> = self.earliest_sorted.lock().await;
        let rev_item = early_lock.peek()?;
        let task = self.tasks.get(&rev_item.1)?;
        Some((task.value().clone(), rev_item.0.clone(), rev_item.1.clone()))
    }

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Arc<ErasedTask>> {
        self.tasks.get(idx).map(|x| x.value().clone())
    }

    async fn pop(&self) {
        let mut early_lock = self.earliest_sorted.lock().await;
        early_lock.pop();
    }

    async fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.tasks.contains_key(idx)
    }

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> Result<(), DynArcError> {
        let task =
            self.tasks
                .get(idx)
                .ok_or(
                    Arc::new(ChronographerErrors::TaskIdentifierNonExistent(format!(
                        "{idx:?}"
                    ))) as DynArcError
                )?;

        let now = clock.now().await;
        let notifier = TriggerNotifier::new::<C>(idx.clone(), self.sender.clone());
        task.trigger().trigger(now, notifier).await
    }

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask,
    ) -> Result<C::TaskIdentifier, DynArcError> {
        let idx = C::TaskIdentifier::generate();
        let task = Arc::new(task);
        self.tasks.insert(idx.clone(), task.clone());
        let now = clock.now().await;
        let notifier = TriggerNotifier::new::<C>(idx.clone(), self.sender.clone());
        task.trigger().trigger(now, notifier).await?;

        Ok(idx)
    }

    async fn remove(&self, idx: &C::TaskIdentifier) {
        self.tasks.remove(idx);
    }

    async fn clear(&self) {
        self.earliest_sorted.lock().await.clear();
        self.tasks.clear();
    }
}
