#[allow(missing_docs)]
pub mod ephemeral; // skipcq: RS-D1001

pub use ephemeral::*;
use std::fmt::Debug;

use crate::clock::SchedulerClock;
use crate::task::Task;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;

/// [`SchedulerTaskStore`] is a trait for implementing a storage mechanism for tasks, it allows
/// for retrieving the earliest task, storing a task with its task schedule, removing a task via
/// an index... etc. This mechanism is used for the [`Scheduler`] struct
#[async_trait]
pub trait SchedulerTaskStore: Debug + Send + Sync {
    /// Retrieves / Peeks the earliest task, without modifying any internal storage
    async fn retrieve(&self) -> Option<(Arc<Task>, SystemTime, usize)>;

    /// Pops the earliest task by modifying any internal storage
    async fn pop(&self);

    /// Checks if an index of a task exists (i.e. The task is registered)
    async fn exists(&self, idx: usize) -> bool;

    /// Reschedules a task instance based on index, it automatically calculates
    /// the new time from the task's schedule
    async fn reschedule(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>, idx: usize);

    /// Stores a task as an entry, returning its index
    async fn store(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>) -> usize;

    /// Removes a task based on an index
    async fn remove(&self, idx: usize);

    /// Clears fully all the contents of the task store
    async fn clear(&self);
}

#[async_trait]
impl<TS: SchedulerTaskStore + 'static> SchedulerTaskStore for Arc<TS> {
    async fn retrieve(&self) -> Option<(Arc<Task>, SystemTime, usize)> {
        self.as_ref().retrieve().await
    }

    async fn pop(&self) {
        self.as_ref().pop().await
    }

    async fn exists(&self, idx: usize) -> bool {
        self.as_ref().exists(idx).await
    }

    async fn reschedule(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>, idx: usize) {
        self.as_ref().reschedule(clock, task, idx).await
    }

    async fn store(&self, clock: Arc<dyn SchedulerClock>, task: Arc<Task>) -> usize {
        self.as_ref().store(clock, task).await
    }

    async fn remove(&self, idx: usize) {
        self.as_ref().remove(idx).await
    }

    async fn clear(&self) {
        self.as_ref().clear().await
    }
}
