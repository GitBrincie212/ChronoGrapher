use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::backend::PersistenceBackend;

/// [`PersistenceObject`] is a trait used for serialization and deserialization (via serde) while
/// also having an associated Persistence ID (Identifier), which is used for tracking the concrete type via
/// a string that is guarantee to be unique. For creating a unique identifier, one recommended format to
/// use is:
/// ```ignore
/// [CRATE]::[TYPE_NAME]#[CUSTOM UUID]
/// ```
/// For UUID generation, we recommend using https://www.uuidgenerator.net/version4.
/// The system is used closely with [`PersistenceBackend`]
///
/// # Supertrait(s)
/// When implementing [`PersistenceObject`], one has to also implement [`Serialize`] and
/// [`Deserialize`] traits, as they are the backbone to what allows serialization and deserialization
/// respectively
///
/// # Object Safety
/// [`PersistenceObject`] is not object safe as it contains an associated constant
///
/// # See Also
/// - [`PersistenceBackend`]
pub trait PersistenceObject: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    const PERSISTENCE_ID: &'static str;
}
