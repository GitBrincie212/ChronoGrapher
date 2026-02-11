pub mod ephemeral;
// skipcq: RS-D1001

pub use ephemeral::*;

use crate::scheduler::SchedulerConfig;
#[allow(unused_imports)]
use crate::task::{ErasedTask, DynArcError};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;

/// [`SchedulerTaskStore`] is a trait for implementing a storage mechanism for tasks, it allows
/// for retrieving the earliest task, storing a task with its task schedule, removing a task via
/// an index... etc. This mechanism is used for the [`Scheduler`] struct
///
/// # Required Method(s)
/// When one plans to implement [`SchedulerTaskStore`], they have to supply implementations for
/// the methods:
/// - [`SchedulerTaskStore::retrieve`] Gets the earliest task (peeks, doesn't pop)
/// - [`SchedulerTaskStore::pop`] Pops off the earliest task
/// - [`SchedulerTaskStore::exists`] Checks if an index corresponds to a task
/// - [`SchedulerTaskStore::reschedule`] Reschedules a task based on the index
/// - [`SchedulerTaskStore::store`] Stores a task as an entry and returns its index
/// - [`SchedulerTaskStore::remove`] Removes a task based on its index
/// - [`SchedulerTaskStore::clear`] Clears fully the task store, all task entries are removed
///
/// # Trait Implementation(s)
/// [`SchedulerTaskStore`] has specifically one implementation present in the library, that being
/// [`EphemeralSchedulerTaskStore`] which is an in-memory task store and does not handle persistence
///
/// # Object Safety
/// [`SchedulerTaskStore`] is object safe as seen throughout the source code of [`Scheduler`]
///
/// # See Also
/// - [`Scheduler`]
/// - [`EphemeralSchedulerTaskStore`]
/// - [`SchedulerTaskStore::retrieve`]
/// - [`SchedulerTaskStore::pop`]
/// - [`SchedulerTaskStore::exists`]
/// - [`SchedulerTaskStore::reschedule`]
/// - [`SchedulerTaskStore::store`]
/// - [`SchedulerTaskStore::remove`]
/// - [`SchedulerTaskStore::clear`]
#[async_trait]
pub trait SchedulerTaskStore<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    /// Retrieves / Peeks the earliest task, without modifying any internal storage
    ///
    /// # Returns
    /// An option collection that contains the current [`Task`], the time it is/was scheduled and
    /// the task's index. This can be ``None`` if there is no early task to retrieve
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`SchedulerTaskStore`]
    async fn retrieve(&self) -> Option<(Arc<ErasedTask>, SystemTime, C::TaskIdentifier)>;

    /// Gets the task based on an index
    ///
    /// # Argument(s)
    /// This method requests one argument, that being the index as ``idx``
    ///
    /// # Returns
    /// An option task where it is ``Some(...)`` if the task exists with this index,
    /// whereas ``None`` if it does not correspond to an index. This index parameter
    /// can be gathered from [`SchedulerTaskStore::store`] and never changes
    ///
    /// # See Also
    /// - [`SchedulerTaskStore`]
    async fn get(&self, idx: &C::TaskIdentifier) -> Option<Arc<ErasedTask>>;

    /// Pops the earliest task by modifying any internal storage. This mechanism
    /// is kept separate from [`SchedulerTaskStore::retrieve`] due to the fact that one might
    /// only want to peek and not pop off the earliest task
    ///
    /// # See Also
    /// - [`SchedulerTaskStore`]
    /// - [`SchedulerTaskStore::retrieve`]
    async fn pop(&self);

    /// Checks if an index of a task exists (i.e. The task is registered)
    ///
    /// # Argument(s)
    /// This method requires only one argument, that being the index as ``idx``,
    /// corresponding to a [`Task`]. This index parameter can be gathered from
    /// [`SchedulerTaskStore::store`] and never changes
    ///
    /// # Returns
    /// A boolean indicating if the index corresponds to a [`Task`] or not
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`SchedulerTaskStore`]
    /// - [`SchedulerTaskStore::store`]
    async fn exists(&self, idx: &C::TaskIdentifier) -> bool;

    /// Reschedules a [`Task`] instance based on index, it automatically calculates
    /// the new time from the task's [`TaskSchedule`]
    ///
    /// # Argument(s)
    /// This method requires 2 arguments, those being the ``clock`` as [`SchedulerClock`]
    /// wrapped in an ``Arc<T>``,and a corresponding index parameter as ``idx``. This index
    /// parameter can be gathered from [`SchedulerTaskStore::store`] and never changes
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`SchedulerClock`]
    /// - [`SchedulerTaskStore`]
    /// - [`SchedulerTaskStore::store`]
    async fn reschedule(
        &self,
        clock: &C::SchedulerClock,
        idx: &C::TaskIdentifier,
    ) -> Result<(), DynArcError>;

    /// Stores a task as an entry, returning its index
    ///
    /// # Argument(s)
    /// This method accepts 2 arguments, those being the [`SchedulerClock`] as ``clock`` wrapped
    /// in an ``Arc<T>`` and the [`Task`] wrapped also in an ``Arc<T>``
    ///
    /// # Returns
    /// The index pointing to the corresponding entry (task), this index cannot change,
    /// as such you can rely upon it throughout the code without any worry. This index
    /// can be used in other methods as a reference to the [`Task`]
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`SchedulerClock`]
    /// - [`SchedulerTaskStore`]
    async fn store(
        &self,
        clock: &C::SchedulerClock,
        task: ErasedTask,
    ) -> Result<C::TaskIdentifier, DynArcError>;

    /// Removes a task based on an index, depending on the implementation,
    /// it may handle differently the case where the index does not exist
    ///
    /// # Argument(s)
    /// This method requests one argument, this being the index which
    /// corresponds to the [`Task`] entry. This index parameter
    /// can be gathered from [`SchedulerTaskStore::store`] and never changes
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`SchedulerTaskStore`]
    async fn remove(&self, idx: &C::TaskIdentifier);

    /// Clears fully all the contents of the task store
    ///
    /// # See Also
    /// - [`SchedulerTaskStore`]
    async fn clear(&self);
}
