use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::task_store::{RescheduleError, SchedulePayload, SchedulerTaskStore};
use crate::task::{ErasedTask, TriggerNotifier};
use async_trait::async_trait;
use dashmap::DashMap;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
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

type EarlyMutexLock<'a, C> = MutexGuard<'a, BinaryHeap<InternalScheduledItem<C>>>;

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    earliest_sorted: Arc<Mutex<BinaryHeap<InternalScheduledItem<C>>>>,
    tasks: DashMap<C::TaskIdentifier, Arc<ErasedTask<C::TaskError>>>,
    sender: tokio::sync::mpsc::Sender<SchedulePayload>,
    notifier: tokio::sync::Notify
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<SchedulePayload>(1024);

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
            notifier: tokio::sync::Notify::default()
        }
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type StoredTask = Arc<ErasedTask<C::TaskError>>;

    async fn init(&self) {}

    async fn retrieve(&self) -> (Self::StoredTask, SystemTime, C::TaskIdentifier) {
        loop {
            let early_lock: EarlyMutexLock<'_, C> = self.earliest_sorted.lock().await;
            if let Some(rev_item) = early_lock.peek()
                && let Some(task) = self.tasks.get(&rev_item.1)
            {
                return (task.value().clone(), rev_item.0, rev_item.1.clone());
            }
            drop(early_lock);
            self.notifier.notified().await;
        }
    }

    fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask> {
        self.tasks.get(idx).map(|x| x.value().clone())
    }

    async fn pop(&self) {
        let mut early_lock = self.earliest_sorted.lock().await;
        early_lock.pop();
    }

    fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.tasks.contains_key(idx)
    }

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> RescheduleError {
        if let Some(task) = self.tasks.get(idx) {
            let now = clock.now();
            let notifier = TriggerNotifier::new::<C>(idx.clone(), self.sender.clone());
            return match task.trigger().trigger(now, notifier).await {
                Ok(()) => {
                    self.notifier.notify_waiters();
                    RescheduleError::Success
                },
                Err(err) => RescheduleError::TriggerError(err)
            }
        }
        RescheduleError::UnknownTask
    }

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        id: C::TaskIdentifier,
        task: ErasedTask<C::TaskError>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let task = Arc::new(task);
        self.tasks.insert(id.clone(), task.clone());
        let now = clock.now();
        let notifier = TriggerNotifier::new::<C>(id.clone(), self.sender.clone());
        task.trigger().trigger(now, notifier).await?;
        self.notifier.notify_waiters();

        Ok::<<C as SchedulerConfig>::TaskIdentifier, Box<dyn Error + Send + Sync>>(id)
    }

    async fn remove(&self, idx: &C::TaskIdentifier) {
        self.tasks.remove(idx);
    }

    async fn clear(&self) {
        self.earliest_sorted.lock().await.clear();
        self.tasks.clear();
    }

    fn iter(&self) -> impl Iterator<Item = (C::TaskIdentifier, Self::StoredTask)> {
        self.tasks.iter().map(|x| (
            x.key().clone(),
            x.value().clone()
        ))
    }
}
