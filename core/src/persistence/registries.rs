use crate::persistence::PersistenceObject;
use dashmap::DashMap;
use erased_serde::Deserializer as ErasedDeserializer;
use erased_serde::Serializer as ErasedSerializer;
use serde::ser::Error as SerializeError;
use std::any::{Any, TypeId};
use std::sync::{Arc, LazyLock};

pub type AnySerializeFunc =
    fn(&dyn Any, &mut dyn ErasedSerializer) -> Result<(), erased_serde::Error>;
pub type AnyDeserializeFunc =
    fn(&mut dyn ErasedDeserializer<'_>) -> Result<Arc<dyn Any>, erased_serde::Error>;

pub static PERSISTENCE_REGISTRIES: LazyLock<PersistenceRegistriesManager> =
    LazyLock::new(|| PersistenceRegistriesManager::default());

#[derive(Clone, Copy)]
pub struct ErasedPersistenceEntry {
    pub serialize: AnySerializeFunc,
    pub deserialize: AnyDeserializeFunc,
}

#[derive(Default)]
pub struct PersistenceRegistriesManager {
    persistent_registry: DashMap<&'static str, ErasedPersistenceEntry>,
    runtime_registry: DashMap<TypeId, &'static str>,
}

impl PersistenceRegistriesManager {
    pub fn register<T: PersistenceObject>(&self) {
        self.persistent_registry.insert(
            T::PERSISTENCE_ID,
            ErasedPersistenceEntry {
                serialize: |any, ser| {
                    let typed = any.downcast_ref::<T>().ok_or_else(|| {
                        erased_serde::Error::custom("Type mismatch during serialization")
                    })?;
                    erased_serde::Serialize::erased_serialize(typed, ser)
                },

                deserialize: |de| {
                    let value: T = erased_serde::deserialize(de)?;
                    Ok(Arc::new(value) as Arc<dyn Any>)
                },
            },
        );

        self.runtime_registry
            .insert(TypeId::of::<T>(), T::PERSISTENCE_ID);
    }

    pub fn get_hook_id(&self, type_id: &TypeId) -> Option<&'static str> {
        let id = self.runtime_registry.get(type_id)?;
        Some(id.value())
    }

    pub fn get_hook_entry(&self, hook_id: &'static str) -> Option<ErasedPersistenceEntry> {
        let entry = self.persistent_registry.get(hook_id)?;
        Some(*entry.value())
    }
}
