use serde_json::Map;
use std::fmt::Debug;
use thiserror::Error;

#[allow(unused_imports)]
use crate::task::SelectTaskFrame;

#[allow(unused_imports)]
use crate::task::ConditionalFrame;

#[allow(unused_imports)]
use crate::task::DependencyTaskFrame;

#[allow(unused_imports)]
use crate::task::TimeoutTaskFrame;

#[allow(unused_imports)]
use crate::task::dependencyframe::DependentFailureOnFail;

/// [`ChronographerErrors`] is the main enum that contains all the errors which can be thrown by
/// ChronoGrapher, it uses under the hood [`thiserror`] to make it as smooth sailing to add more
/// errors in the future as possible. This enum is private only to the core package itself,
/// as no other is aware of the existence
#[derive(Error, Debug)]
pub enum ChronographerErrors {
    /// This error is meant to happen when retrieving an index from a container that has a
    /// specified length, but the index is out of bounds. In the core package it is mainly caused
    /// by [`SelectTaskFrame`]
    #[error(
        "Task frame index `{0}` is out of bounds for `{1}` with task frame size `{2}` element(s)"
    )]
    TaskIndexOutOfBounds(usize, String, usize),

    /// This error is meant to happen when [`ConditionalTaskFrame`]
    /// returns true and the flag ``error_on_false`` is set to true
    #[error(
        "ConditionalTaskFrame returned false with error_on_false set to true, as such this error returns"
    )]
    TaskConditionFail,

    /// This error is meant to happen when dependencies from [`DependencyTaskFrame`]
    /// aren't resolved with the dependent behavior set to [`DependentFailureOnFail`]
    #[error("Dependencies have not been resolved")]
    TaskDependenciesUnresolved,

    /// This error is meant to happen when a timeout occurs on [`TimeoutTaskFrame`], i.e.
    /// if a threshold of time counts down fully to zero but the task frame hasn't completed
    #[error("`{0}` Timed out")]
    TimeoutError(String),

    /// This error is meant to happen when an insertion happens when a
    /// field already exists (modifying basically the ObserverField which is non-idiomatic)
    #[error("`{0}` already exists as a field, grab the observer as opposed to fully modifying")]
    DynamicFieldAlreadyExists(String),

    /// This error is meant to happen when deserialization of a system (for task) fails.
    /// The first field is the type that field, the second is the error message and the third
    /// is the JSON payload
    #[error(
        "Deserialization of `{0}` failed, with an error message `{1}` and the payload being:\n{2:?}"
    )]
    DeserializationFailed(String, String, Map<String, serde_json::Value>),

    /// This error is meant to happen when deserialization of a system (for task) detects
    /// this JSON payload is not an object
    #[error(
        "Deserialization of `{0}` failed, as this is not a json object, rather a standalone value"
    )]
    NonObjectDeserialization(String, serde_json::Value),

    /// This error is meant to happen when serialization occurs
    /// on a non-persistent critical component
    #[error(
        "Serialization of `{0}` failed, as this is a critical component and persistence \
        is required to deserialize this form back"
    )]
    NonPersistentObject(String),

    /// This error originates when an object isn't on the specific retrieve register
    #[error("Deserialization of `{0}` failed, as this is not recognised component")]
    NonMatchingIDs(String)
}
