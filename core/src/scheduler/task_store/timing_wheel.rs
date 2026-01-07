use crate::errors::ChronographerErrors;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::scheduler::timing_wheel_core::TimingWheelCore;
use crate::task::{ErasedTask, TaskError};
use crate::utils::{TaskIdentifier, date_time_to_system_time, system_time_to_date_time};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};

/// A timing wheel implementation for efficient task scheduling
///
/// This implementation provides O(1) insertion and removal operations,
/// making it ideal for scenarios with millions of concurrent tasks.
///
/// # Algorithm Overview
/// The timing wheel maintains an array of slots, where each slot contains
/// a list of tasks scheduled to execute at approximately the same time.
/// The wheel advances at fixed intervals (ticks), processing tasks in the
/// current slot.
///
/// # Performance Characteristics
/// - **Insert**: O(1)
/// - **Remove**: O(1)
/// - **Peek**: O(1) (amortized)
/// - **Space**: O(n) where n is number of active tasks
///
/// # Trade-offs
/// - Tasks may execute up to `interval` duration later than scheduled
/// - Memory usage is proportional to `num_slots` regardless of task count
/// - Not suitable for tasks requiring precise timing
pub struct TimingWheelSchedulerTaskStore<C>
where
    C: SchedulerConfig<Timestamp = SystemTime>,
{
    core: TimingWheelCore<C::TaskIdentifier, ErasedTask>,
}

impl<C> TimingWheelSchedulerTaskStore<C>
where
    C: SchedulerConfig<Timestamp = SystemTime>,
{
    /// Create a new timing wheel with the specified interval and number of slots
    ///
    /// # Arguments
    /// * `interval` - Time between wheel ticks (e.g., 1 second)
    /// * `num_slots` - Number of slots in the wheel (must be power of 2 for optimal performance)
    ///
    /// # Panics
    /// Panics if `num_slots` is 0
    pub fn new(interval: Duration, num_slots: usize) -> Self {
        Self {
            core: TimingWheelCore::new(interval, num_slots),
        }
    }
}

impl<C> Default for TimingWheelSchedulerTaskStore<C>
where
    C: SchedulerConfig<Timestamp = SystemTime>,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(1), 3600)
    }
}

#[async_trait]
impl<C> SchedulerTaskStore<C> for TimingWheelSchedulerTaskStore<C>
where
    C: SchedulerConfig<Timestamp = SystemTime>,
{
    async fn retrieve(&self) -> Option<(Arc<ErasedTask>, C::Timestamp, C::TaskIdentifier)> {
        self.core.peek_ready().await
    }

    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Arc<ErasedTask>> {
        self.core.get(idx).await
    }

    async fn pop(&self) {
        if let Some((_, _, id)) = self.core.peek_ready().await {
            self.core.remove(&id).await;
        }
    }

    async fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.core.exists(idx).await
    }

    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> Result<(), TaskError> {
        let timestamp_now = clock.now().await;
        if let Some(task) = self.core.get(idx).await {
            let now = system_time_to_date_time(&timestamp_now);
            let future_time = task.schedule().next_after(&now)?;
            let sys_future_time = date_time_to_system_time(future_time);
            self.core
                .move_task(idx.clone(), sys_future_time)
                .await
                .map_err(|_| {
                    Arc::new(ChronographerErrors::TaskIdentifierNonExistent(format!(
                        "{:?}",
                        idx
                    ))) as Arc<dyn Debug + Send + Sync>
                })
        } else {
            Err(
                Arc::new(ChronographerErrors::TaskIdentifierNonExistent(format!(
                    "{:?}",
                    idx
                ))) as Arc<dyn Debug + Send + Sync>,
            )
        }
    }

    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask,
    ) -> Result<C::TaskIdentifier, TaskError> {
        let timestamp_now = clock.now().await;
        let now = system_time_to_date_time(&timestamp_now);
        let future_time = task.schedule().next_after(&now)?;
        let sys_future_time = date_time_to_system_time(future_time);

        let id = C::TaskIdentifier::generate();
        let _ = self
            .core
            .insert(id.clone(), Arc::new(task), sys_future_time)
            .await;
        Ok(id)
    }

    async fn remove(&self, idx: &C::TaskIdentifier) {
        self.core.remove(idx).await;
    }

    async fn clear(&self) {
        self.core.clear().await;
    }
}
