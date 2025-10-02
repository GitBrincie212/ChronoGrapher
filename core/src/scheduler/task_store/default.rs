use crate::backend::PersistenceBackend;
use crate::clock::SchedulerClock;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::Task;
use crate::utils::{date_time_to_system_time, system_time_to_date_time};
use async_trait::async_trait;
use dashmap::DashMap;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::SystemTime;
use tokio::sync::Mutex;

struct DefaultScheduledItem(Arc<Task>, SystemTime, usize);

impl Eq for DefaultScheduledItem {}

impl PartialEq<Self> for DefaultScheduledItem {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd<Self> for DefaultScheduledItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for DefaultScheduledItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

/// [`DefaultSchedulerTaskStore`] is an implementation of [`SchedulerTaskStore`]
/// that can operate in-memory and persistence (can be configured with a [`PersistenceBackend`])
///
/// # Usage Note(s)
/// By default [`DefaultSchedulerTaskStore`] operates in-memory,
/// it doesn't store any information on the disk, while being fast, it makes it brittle
/// to crashes and shutdowns. For enterprise use, it is advised to configure a
/// backend. It is mostly meant to be used for demos or for debugging (where one
/// doesn't care to persist information on disk)
///
/// # Constructor(s)
/// When constructing a new [`DefaultSchedulerTaskStore`], one can use
/// [`DefaultSchedulerTaskStore::ephemeral`] for ephemeral-only (in-memory) storage or
/// [`DefaultSchedulerTaskStore<T>::persistent`] for backend storage with a provided
/// backend
///
/// # Trait Implementation(s)
/// [`DefaultSchedulerTaskStore`] obviously implements the [`SchedulerTaskStore`]
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
/// - [`DefaultSchedulerTaskStore::new`]
pub struct DefaultSchedulerTaskStore<T: PersistenceBackend = ()> {
    earliest_sorted: Mutex<BinaryHeap<Reverse<DefaultScheduledItem>>>,
    tasks: DashMap<usize, Arc<Task>>,
    id: AtomicUsize,
    backend: T,
}

impl<T: PersistenceBackend> Debug for DefaultSchedulerTaskStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "EphemeralDefaultTaskStore [{:?}]",
            self.earliest_sorted
                .blocking_lock()
                .iter()
                .rev()
                .map(|x| x.0.0.clone())
                .collect::<Vec<_>>()
        ))
    }
}

impl DefaultSchedulerTaskStore {
    /// Creates / Constructs a new [`DefaultSchedulerTaskStore`] instance which
    /// only operates in-memory, one can construct a version for persistence
    /// use, via [`DefaultSchedulerTaskStore<T>::persistent`]
    ///
    /// # Returns
    /// The newly constructed [`DefaultSchedulerTaskStore`] wrapped in an ``Arc<T>``.
    ///
    /// # See Also
    /// - [`DefaultSchedulerTaskStore`]
    /// - [`DefaultSchedulerTaskStore::default`]
    pub fn ephemeral() -> Arc<Self> {
        Arc::new(Self {
            earliest_sorted: Mutex::new(BinaryHeap::new()),
            tasks: DashMap::new(),
            id: AtomicUsize::new(0),
            backend: (),
        })
    }
}

impl<T: PersistenceBackend> DefaultSchedulerTaskStore<T> {
    /// Creates / Constructs a new [`DefaultSchedulerTaskStore`] instance which
    /// operates in-memory and stores information to be reconstructed at runtime
    /// when a crash occurs. One can also construct a variant for only in-memory use
    /// via [`DefaultSchedulerTaskStore::ephemeral`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, this being the [`PersistenceBackend`]
    /// implementation to use
    ///
    /// # Returns
    /// The newly constructed [`DefaultSchedulerTaskStore`] wrapped in an ``Arc<T>``.
    ///
    /// # See Also
    /// - [`DefaultSchedulerTaskStore`]
    /// - [`DefaultSchedulerTaskStore::ephemeral`]
    pub fn persistent(backend: T) -> Arc<Self> {
        Arc::new(Self {
            earliest_sorted: Mutex::new(BinaryHeap::new()),
            tasks: DashMap::new(),
            id: AtomicUsize::new(0),
            backend,
        })
    }
}

#[async_trait]
impl<T: PersistenceBackend> SchedulerTaskStore for DefaultSchedulerTaskStore<T> {
    async fn retrieve(&self) -> Option<(Arc<Task>, SystemTime, usize)> {
        let early_lock = self.earliest_sorted.lock().await;
        let rev_item = early_lock.peek()?;
        let item = &rev_item.0;
        Some((item.0.clone(), item.1, item.2))
    }

    async fn get(&self, idx: &usize) -> Option<Arc<Task>> {
        self.tasks.get(idx).map(|x| x.value().clone())
    }

    async fn pop(&self) {
        self.earliest_sorted.lock().await.pop();
    }

    async fn exists(&self, idx: &usize) -> bool {
        self.tasks.contains_key(idx)
    }

    async fn reschedule(&self, clock: Arc<dyn SchedulerClock>, idx: &usize) {
        let sys_now = clock.now().await;
        let task = self.tasks.get(idx).unwrap();
        let now = system_time_to_date_time(sys_now);
        let future_time = task.schedule().next_after(&now).unwrap();
        let sys_future_time = date_time_to_system_time(future_time);

        let mut lock = self.earliest_sorted.lock().await;
        lock.push(Reverse(DefaultScheduledItem(
            task.clone(),
            sys_future_time,
            *idx,
        )));
    }

    async fn store(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>) -> usize {
        let sys_last_exec = clock.now().await;
        let last_exec = system_time_to_date_time(sys_last_exec);
        let future_time = task.schedule().next_after(&last_exec).unwrap();
        let idx: usize = {
            let idx = self.id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.tasks.insert(idx, task.clone());
            idx
        };
        let sys_future_time = SystemTime::from(future_time);
        let entry = DefaultScheduledItem(task, sys_future_time, idx);
        let mut earliest_tasks = self.earliest_sorted.lock().await;
        earliest_tasks.push(Reverse(entry));

        idx
    }

    async fn remove(&self, idx: &usize) {
        self.tasks.remove(idx);
    }

    async fn clear(&self) {
        self.earliest_sorted.lock().await.clear();
        self.tasks.clear();
    }
}
