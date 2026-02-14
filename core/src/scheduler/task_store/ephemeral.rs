use crate::errors::StandardCoreErrorsCG;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{DynArcError, ErasedTask, TriggerNotifier};
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

type EarlyMutexLock<'a, C> = MutexGuard<'a, BinaryHeap<InternalScheduledItem<C>>>;

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    earliest_sorted: Arc<Mutex<BinaryHeap<InternalScheduledItem<C>>>>,
    tasks: DashMap<C::TaskIdentifier, Arc<ErasedTask<C::Error>>>,
    sender:
        tokio::sync::mpsc::Sender<(Box<dyn Any + Send + Sync>, Result<SystemTime, DynArcError>)>,
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
    type StoredTask = Arc<ErasedTask<C::Error>>;

    async fn init(&self) {}

    async fn retrieve(&self) -> Option<(Self::StoredTask, SystemTime, C::TaskIdentifier)> {
        let early_lock: EarlyMutexLock<'_, C> = self.earliest_sorted.lock().await;
        let rev_item = early_lock.peek()?;
        let task = self.tasks.get(&rev_item.1)?;
        Some((task.value().clone(), rev_item.0, rev_item.1.clone()))
    }

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask> {
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
                    Arc::new(StandardCoreErrorsCG::TaskIdentifierNonExistent(format!(
                        "{idx:?}"
                    ))) as DynArcError,
                )?;

        let now = clock.now().await;
        let notifier = TriggerNotifier::new::<C>(idx.clone(), self.sender.clone());
        task.trigger().trigger(now, notifier).await
    }

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask<C::Error>,
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
