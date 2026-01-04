use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ErasedTask, TaskError};
use crate::utils::{date_time_to_system_time, system_time_to_date_time, TaskIdentifier};
use async_trait::async_trait;
use dashmap::DashMap;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, MutexGuard};
use crate::errors::ChronographerErrors;
use crate::scheduler::SchedulerConfig;

struct InternalScheduledItem<C: SchedulerConfig>(C::Timestamp, C::TaskIdentifier);

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
    earliest_sorted: Mutex<BinaryHeap<InternalScheduledItem<C>>>,
    tasks: DashMap<C::TaskIdentifier, Arc<ErasedTask>>,
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        Self {
            earliest_sorted: Mutex::new(BinaryHeap::new()),
            tasks: DashMap::new(),
        }
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    async fn retrieve(&self) -> Option<(Arc<ErasedTask>, C::Timestamp, C::TaskIdentifier)> {
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

    async fn reschedule(&self, clock: &C::SchedulerClock, idx: &C::TaskIdentifier) -> Result<(), TaskError> {
        let timestamp_now = clock.now().await;
        let task = self.tasks
            .get(idx)
            .ok_or(
                Arc::new(ChronographerErrors::TaskIdentifierNonExistent) as Arc<dyn Debug + Send + Sync>
            )?;
        let now = system_time_to_date_time(&timestamp_now);
        let future_time = task.schedule()
            .next_after(&now)?;
        let sys_future_time = date_time_to_system_time(future_time);

        let mut lock = self.earliest_sorted.lock().await;
        lock.push(InternalScheduledItem(sys_future_time, *idx));
        Ok(())
    }

    async fn store(&self, clock: &C::SchedulerClock, task: ErasedTask) -> Result<C::TaskIdentifier, TaskError> {
        let last_exec_timestamp = clock.now().await;
        let last_exec = system_time_to_date_time(&last_exec_timestamp);
        let future_time = task.schedule().next_after(&last_exec)?;
        let idx = C::TaskIdentifier::generate();
        self.tasks.insert(idx.clone(), Arc::new(task));
        let sys_future_time = SystemTime::from(future_time);
        let entry = InternalScheduledItem(sys_future_time, idx.clone());
        let mut earliest_tasks = self.earliest_sorted.lock().await;
        earliest_tasks.push(entry);

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
