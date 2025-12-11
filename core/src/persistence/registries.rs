use std::any::{Any, TypeId};
use std::sync::{Arc, LazyLock};
use dashmap::DashMap;
use erased_serde::Serializer as ErasedSerializer;
use erased_serde::Deserializer as ErasedDeserializer;
use serde::{Serialize, Serializer};
use serde::ser::{Error as SerializeError, SerializeStruct};
use crate::persistence::PersistenceObject;

pub static PERSISTENCE_REGISTRIES: LazyLock<PersistenceRegistriesManager> = LazyLock::new(|| {
    PersistenceRegistriesManager::default()
});

pub struct ErasedPersistenceEntry {
    pub serialize: fn(&dyn Any, &mut dyn ErasedSerializer) -> Result<(), erased_serde::Error>,
    pub deserialize: fn(&mut dyn ErasedDeserializer<'_>) -> Result<Arc<dyn Any>, erased_serde::Error>,
}

#[derive(Default)]
pub struct PersistenceRegistriesManager {
    persistent_registry: DashMap<&'static str, ErasedPersistenceEntry>,
    runtime_registry: DashMap<TypeId, &'static str>,
}

impl PersistenceRegistriesManager {
    pub fn register<T: PersistenceObject + 'static>(&self) {
        self.persistent_registry.insert(T::PERSISTENCE_ID, ErasedPersistenceEntry {
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
        });

        self.runtime_registry.insert(TypeId::of::<T>(), T::PERSISTENCE_ID);
    }
}