use crate::task::ErasedTask;
use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[allow(unused_imports)]
use crate::persistence::PersistenceObject;

pub struct PersistPath(String);

impl PersistPath {
    pub fn extend(&self, path: &'_ str) -> Self {
        Self(self.0.clone() + "/" + path)
    }
}

impl Into<String> for PersistPath {
    fn into(self) -> String {
        self.0
    }
}

/// [`PersistenceBackend`] is the mechanism that allows transfer a runtime system
/// to disk and safely load back from it. The format of the storage, the way it is stored and
/// the way it is saved and loaded is dictated by this mechanism
///
/// It requires for both saving and loading a [`PersistenceObject`] which is both serializable and
/// deserializable, while also having an associated ID attached to it (that is guarantee to be unique)
///
/// # Required Method(s)
/// [`PersistenceBackend`]
///
/// # Trait Implementation(s)
/// There are various implementations of [`PersistenceBackend`], one such type that implements it
/// is ``()``, which does not handle storage of anything (it is used as a wildcard basically)
///
/// # Object Safety
/// [`PersistenceBackend`] is not object safe as it uses generics in its methods
///
/// # See Also
/// - [`PersistenceObject`]
#[async_trait]
pub trait PersistenceBackend: Send + Sync + 'static {
    async fn save_task(&self, task: &ErasedTask);
    async fn load_task(&self, id: Uuid) -> Option<Arc<ErasedTask>>;
    async fn stored_task_ids(&self) -> Vec<Uuid>;
    async fn update_field<T: Serialize>(&self, path: PersistPath, field: &T);
}

#[async_trait]
impl PersistenceBackend for () {
    async fn save_task(&self, _task: &ErasedTask) {}
    async fn load_task(&self, _id: Uuid) -> Option<Arc<ErasedTask>> {
        None
    }
    async fn stored_task_ids(&self) -> Vec<Uuid> {
        vec![]
    }
    async fn update_field<T: Serialize>(&self, _path: PersistPath, _field: &T) {}
}
