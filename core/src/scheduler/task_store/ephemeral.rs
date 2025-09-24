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

pub(crate) struct EphemeralScheduledItem(
    pub(crate) Arc<Task>,
    pub(crate) SystemTime,
    pub(crate) usize,
);

impl Eq for EphemeralScheduledItem {}

impl PartialEq<Self> for EphemeralScheduledItem {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd<Self> for EphemeralScheduledItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for EphemeralScheduledItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

pub struct EphemeralDefaultTaskStore {
    earliest_sorted: Mutex<BinaryHeap<Reverse<EphemeralScheduledItem>>>,
    tasks: DashMap<usize, Arc<Task>>,
    id: AtomicUsize,
}

impl Debug for EphemeralDefaultTaskStore {
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

impl EphemeralDefaultTaskStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            earliest_sorted: Mutex::new(BinaryHeap::new()),
            tasks: DashMap::new(),
            id: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl SchedulerTaskStore for EphemeralDefaultTaskStore {
    async fn retrieve(&self) -> Option<(Arc<Task>, SystemTime, usize)> {
        let early_lock = self.earliest_sorted.lock().await;
        let rev_item = early_lock.peek()?;
        let item = &rev_item.0;
        Some((item.0.clone(), item.1, item.2))
    }

    async fn pop(&self) {
        self.earliest_sorted.lock().await.pop();
    }

    async fn exists(&self, idx: usize) -> bool {
        self.tasks.contains_key(&idx)
    }

    async fn reschedule(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>, idx: usize) {
        let sys_now = clock.now().await;
        let now = system_time_to_date_time(sys_now);
        let future_time = task.schedule().next_after(&now).unwrap();
        let sys_future_time = date_time_to_system_time(future_time);

        let mut lock = self.earliest_sorted.lock().await;
        lock.push(Reverse(EphemeralScheduledItem(task, sys_future_time, idx)));
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
        let entry = EphemeralScheduledItem(task, sys_future_time, idx);
        let mut earliest_tasks = self.earliest_sorted.lock().await;
        earliest_tasks.push(Reverse(entry));

        idx
    }

    async fn remove(&self, idx: usize) {
        self.tasks.remove(&idx);
    }

    async fn clear(&self) {
        self.earliest_sorted.lock().await.clear();
        self.tasks.clear();
    }
}
