use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::ErasedTask;
use std::error::Error;
use std::sync::Arc;
use crossbeam::utils::CachePadded;
use slotmap::{new_key_type, SlotMap};

new_key_type! {struct SlotTaskKey;}

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub struct TaskKey {
    shard_idx: u16,
    inner: SlotTaskKey,
}

impl Into<usize> for TaskKey {
    fn into(self) -> usize {
        self.inner.0.as_ffi() as usize
    }
}

#[repr(transparent)]
pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig>(
    Box<[CachePadded<parking_lot::RwLock<SlotMap<SlotTaskKey, Arc<ErasedTask<C::TaskError>>>>>]>
);

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism()
            .unwrap()
            .get();

        let shard_count = (parallelism * 4).next_power_of_two();
        let shards = (0..shard_count)
            .map(|_| CachePadded::new(parking_lot::RwLock::new(SlotMap::default())))
            .collect::<Vec<_>>();

        Self(shards.into_boxed_slice())
    }
}

impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type Key = TaskKey;

    fn get(&self, key: &Self::Key) -> Option<Arc<ErasedTask<C::TaskError>>> {
        let shard = self.0.get(key.shard_idx as usize)?.read();
        Some(shard.get(key.inner)?.clone())
    }

    fn exists(&self, key: &Self::Key) -> bool {
        if let Some(shard) = self.0.get(key.shard_idx as usize){
            return shard.read().contains_key(key.inner)
        }
        false
    }

    fn store(&self, task: Arc<ErasedTask<C::TaskError>>) -> Result<Self::Key, Box<dyn Error + Send + Sync>> {
        let shard_idx = fastrand::u16(0..self.0.len() as u16);
        let inner = self.0[shard_idx as usize].write().insert(task);

        Ok(TaskKey {
            shard_idx,
            inner,
        })
    }

    fn remove(&self, key: &Self::Key) {
        if let Some(shard) = self.0.get(key.shard_idx as usize){
            shard.write().remove(key.inner);
        }
    }

    fn clear(&self) {
        for shard in self.0.iter() {
            shard.write().clear();
        }
    }
}