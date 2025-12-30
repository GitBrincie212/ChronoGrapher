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

/// Global persistence registries manager that holds type-erased serialization and deserialization
/// functions for all registered [`PersistenceObject`] types.
///
/// This static provides thread-safe access to registry mappings between persistence IDs, type IDs,
/// and their corresponding serialization functions.
///
/// # See Also
/// - [`PersistenceRegistriesManager`]
/// - [`PersistenceObject`]
/// - [`ErasedPersistenceEntry`]
pub static PERSISTENCE_REGISTRIES: LazyLock<PersistenceRegistriesManager> =
    LazyLock::new(PersistenceRegistriesManager::default);

/// Type-erased persistence entry containing serialization and deserialization functions.
///
/// This struct holds function pointers that can serialize and deserialize values of a specific
/// type without carrying generic type parameters, enabling runtime type registration and
/// dynamic dispatch for persistence operations.
///
/// # Struct Field(s)
/// - `serialize`: Function pointer for type-erased serialization operations
/// - `deserialize`: Function pointer for type-erased deserialization operations
///
/// # Trait Implementation(s)
/// - [`Clone`]: Allows copying the function pointers
/// - [`Copy`]: Enables bitwise copying since function pointers are trivially copyable
///
/// # Constructor(s)
/// This struct is typically constructed internally by [`PersistenceRegistriesManager::register`]
/// when registering a new [`PersistenceObject`] type.
///
/// # See Also
/// - [`AnySerializeFunc`]
/// - [`AnyDeserializeFunc`]
/// - [`PersistenceRegistriesManager`]
#[derive(Clone, Copy)]
pub struct ErasedPersistenceEntry {
    pub serialize: AnySerializeFunc,
    pub deserialize: AnyDeserializeFunc,
}

/// Manager for persistence registries that handles type registration and lookup.
///
/// This struct maintains two internal registries:
/// - A persistent registry that maps persistence IDs to their serialization/deserialization functions
/// - A runtime registry that maps type IDs to persistence IDs for reverse lookups
///
/// The manager enables dynamic registration of [`PersistenceObject`] types at runtime and provides
/// efficient lookup mechanisms for both serialization and deserialization operations.
///
/// # Struct Field(s)
/// - `persistent_registry`: Maps persistence IDs to [`ErasedPersistenceEntry`] instances
/// - `runtime_registry`: Maps type IDs to persistence ID strings
///
/// # Trait Implementation(s)
/// - [`Default`]: Creates a new manager with empty registries
///
/// # Constructor(s)
/// - [`Default::default()`]: Creates an empty manager ready for type registration
///
/// # See Also
/// - [`PERSISTENCE_REGISTRIES`]
/// - [`ErasedPersistenceEntry`]
/// - [`PersistenceObject`]
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
