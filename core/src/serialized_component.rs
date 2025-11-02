#[allow(unused_imports)]
use crate::backend::PersistenceBackend;
use crate::persistence::PersistentObject;
use serde::{Deserialize, Serialize};

/// [`SerializedComponent`] is a container that wraps a **unique** identifier (ID) corresponding
/// to a concrete type and an IR (Intermediate Representation) as JSON that is not type-safe but
/// represents an object in a [`PersistenceBackend`] agnostic way
///
/// # Constructor(s)
/// When constructing [`SerializedComponent`], the straightforward and commonly used approach is
/// to use [`SerializedComponent::new`] where one can supply their type and the JSON representation
/// of the object they serialized, alternatively they can also use [`SerializedComponent::new_with`],
/// however the user has to supply their own ID **which have to be certain that it is unique for this
/// type**
///
/// # Trait Implementation(s)
/// [`SerializedComponent`] only implements the [`Clone`] trait
///
/// # Cloning Semantics
/// [`SerializedComponent`] deeply clones the intermediate representation as well as the ID
///
/// # See Also
/// - [`PersistenceBackend`]
/// - [`SerializedComponent::new`]
/// - [`SerializedComponent::new_with`]
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializedComponent {
    id: String,
    json: serde_json::Value,
}

impl SerializedComponent {
    /// Creates / Constructs a new [`SerializedComponent`] instance with an ID
    /// controlled by the user, the user must ensure the identifier is **unique** for that type
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being the identifier and the JSON payload which is
    /// the intermediate representation of the instance
    ///
    /// # Returns
    /// The constructed [`SerializedComponent`] instance
    ///
    /// # See Also
    /// - [`SerializedComponent`]
    pub fn new<T: PersistentObject>(json: serde_json::Value) -> Self {
        Self {
            id: T::persistence_id().to_string(),
            json,
        }
    }

    /// Returns the identifier of the type
    pub fn id(&self) -> &'_ str {
        &self.id
    }

    /// Returns the **Intermediate Representation (IR)** in the form of JSON
    pub fn into_ir(self) -> serde_json::Value {
        self.json
    }
}
