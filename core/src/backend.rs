use crate::persistence::PersistenceObject;
use crate::task::TaskError;

/// [`PersistenceBackend`] is the mechanism that allows transfer from a runtime system
/// to a disk and safely load back from it. The format of the storage, the way it is stored and
/// the way it is saved and loaded is dictated by this mechanism
///
/// It requires for both saving and loading a [`PersistenceObject`] which is both serializable and
/// deserializable, while also having an associated ID attached to it (that is guarantee to be unique)
///
/// # Required Method(s)
/// [`PersistenceBackend`] requires an implementation of [`PersistenceBackend::save`] and [`PersistenceBackend::load`].
/// The former handles saving to disk the object, while the latter handles loading from disk the data
/// associated with the object and transforming it to a concrete type back
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
pub trait PersistenceBackend: Send + Sync {
    fn save<T: PersistenceObject>(&self, object: T);
    fn load<T: PersistenceObject>(&self) -> Result<Option<T>, TaskError>;
    fn init(&self) {}
}

impl PersistenceBackend for () {
    fn save<T: PersistenceObject>(&self, _object: T) {}
    fn load<T: PersistenceObject>(&self) -> Result<Option<T>, TaskError> {
        Ok(None)
    }
}