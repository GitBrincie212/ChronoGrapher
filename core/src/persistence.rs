pub mod backend; // skipcq: RS-D1001

pub use backend::PersistPath;
pub use backend::PersistenceBackend;

use erased_serde::Serialize as ErasedSerialized;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct PersistenceContext(
    pub(crate) fn(PersistPath, &dyn erased_serde::Serialize) -> Pin<Box<dyn Future<Output = ()>>>,
);

impl PersistenceContext {
    pub async fn update_field(&self, path: PersistPath, value: &dyn ErasedSerialized) {
        self.0(path, value).await;
    }
}

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
/// # Required Method(s)
/// When implementing the [`PersistenceObject`], one has to supply an implementation
/// to the method [`PersistenceObject::inject_context`] which notifies what the communication layer
/// is between each [`PersistentTracker`] field and the backend via a [`PersistenceContext`] object
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
pub trait PersistenceObject: Serialize + Deserialize<'static> + Send + Sync {
    const PERSISTENCE_ID: &'static str;

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}
