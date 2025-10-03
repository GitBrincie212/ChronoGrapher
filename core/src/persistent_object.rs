use crate::serialized_component::SerializedComponent;
use crate::task::TaskError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::fmt::Debug;
use std::sync::Arc;
#[allow(unused_imports)]
use crate::task::*;

#[allow(unused_imports)]
use crate::backend::PersistenceBackend;

/// [`PersistenceCapability`] is a small utility for controlling the purpose of an object,
///
pub enum PersistenceCapability<'a> {
    NotPersistable,
    Ephemeral,
    Persistable(&'a dyn PersistentObjectDyn),
}

/// [`PersistentObject`] is a trait for implementing a persistent object, this trait
/// is used throughout most task composites, such as [`TaskFrame`], [`TaskEvent`],
/// [`TaskMetadata`]... etc. This system is made to be backend agnostic
///
/// # Required Method(s)
/// When implementing the [`PersistentObject`], one must implement 2 methods, those
/// being [`PersistentObject::store`] and [`PersistentObject::retrieve`], the former
/// is used to serialize into a [`SerializedComponent`] which is an intermediate representation
/// of the type, for the [`PersistenceBackend`] to handle accordingly the full serialization. While
/// the latter is used for deserialization where it accepts the intermediate representation (the
/// way it knows this representation corresponds to the type is via the [`SerializedComponent::id`]
/// and the deserializer register system).
///
/// # Trait Implementation(s)
/// Any type that implements [`Serialize`] and [`Deserialize`] from serde, automatically implements
/// this trait (as such, this integration is handy). In addition, most provided implementations of
/// various systems by ChronoGrapher implement the [`PersistentObject`] trait
///
/// # See Also
/// - [`TaskFrame`]
/// - [`TaskMetadata`]
/// - [`TaskEvent`]
/// - [`SerializedComponent`]
/// - [`PersistenceBackend`]
#[async_trait]
pub trait PersistentObject: Send + Sync {
    fn persistence_id() -> &'static str {
        use once_cell::sync::OnceCell;
        static CELL: OnceCell<String> = OnceCell::new();
        CELL.get_or_init(|| type_name::<Self>().to_string())
            .as_str()
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError>;
    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError>
    where
        Self: Sized;
}

#[async_trait]
pub trait PersistentObjectDyn: Send + Sync {
    async fn persist(&self) -> Result<SerializedComponent, TaskError>;
}

#[async_trait]
impl<T: PersistentObject> PersistentObjectDyn for T {
    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        self.persist().await
    }
}

#[async_trait]
impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync> PersistentObject for T {
    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        match serde_json::to_value(self) {
            Ok(res) => Ok(SerializedComponent::new::<T>(res)),
            Err(err) => Err(Arc::new(err)),
        }
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        serde_json::from_value::<Self>(component.into_ir())
            .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)
    }
}

#[async_trait]
pub trait AsPersistent {
    async fn as_persistent(&self) -> PersistenceCapability {
        PersistenceCapability::NotPersistable
    }
}

#[async_trait]
impl<T: ?Sized> AsPersistent for T {}