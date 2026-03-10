use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::ErasedTask;
use async_trait::async_trait;
use dashmap::DashMap;
use std::error::Error;
use std::sync::Arc;

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig>(DashMap<C::TaskIdentifier, Arc<ErasedTask<C::TaskError>>>);

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        Self(DashMap::new())
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type StoredTask = Arc<ErasedTask<C::TaskError>>;

    async fn init(&self) {}

    fn get(&self, idx: &C::TaskIdentifier) -> Option<Self::StoredTask> {
        self.0.get(idx).map(|x| x.value().clone())
    }

    fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.0.contains_key(idx)
    }

    fn store(
        &self,
        id: &C::TaskIdentifier,
        task: ErasedTask<C::TaskError>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.0.insert(id.clone(), Arc::new(task));
        Ok(())
    }

    fn remove(&self, idx: &C::TaskIdentifier) {
        self.0.remove(idx);
    }

    fn clear(&self) {
        self.0.clear();
    }
}
