use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::TaskError;
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// [`FlagDependency`] is a dependency which can be enabled and disabled from outside, essentially
/// acting more as a checkbox
///
/// # Constructor(s)
/// When constructing a [`FlagDependency`], one can use [`FlagDependency::new`] with a supplied
/// ``Arc<AtomicBool`` acting as the flag which can be changed from the outside
///
/// # Trait Implementation(s)
/// It is clear as day that [`FlagDependency`] implements the [`FrameDependency`] trait,
/// but it also implements the extension traits [`ResolvableFrameDependency`] and [`UnresolvableFrameDependency`]
/// for developers to manually resolve and unresolve
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use chronographer_core::task::dependency::{FlagDependency, FrameDependency};
///
/// let val = Arc::new(AtomicBool::new(false));
///
/// // Creating the dependency
/// let flag = FlagDependency::new(val.clone());
///
/// // ... Some time passes after creation ...
///
/// val.store(true, Ordering::Relaxed);
/// assert!(flag.is_resolved());
/// ```
///
/// # See Also
/// - [`FrameDependency`]
/// - [`FlagDependency::new`]
/// - [`ResolvableFrameDependency`]
/// - [`UnresolvableFrameDependency`]
pub struct FlagDependency(Arc<AtomicBool>, Arc<AtomicBool>);

impl FlagDependency {
    /// Creates / Constructs a new [`FlagDependency`] instance
    ///
    /// # Argument(s)
    /// This method accepts one single argument, that being ``flag``, which is
    /// an atomic boolean wrapped in ``Arc<T>``
    ///
    /// # Returns
    /// The newly created [`FlagDependency`] with the flag being ``flag``
    ///
    /// # See Also
    /// - [`FlagDependency`]
    pub fn new(flag: Arc<AtomicBool>) -> Self {
        Self(flag, Arc::new(AtomicBool::new(true)))
    }
}

#[async_trait]
impl FrameDependency for FlagDependency {
    async fn is_resolved(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    async fn disable(&self) {
        self.1.store(false, Ordering::Relaxed);
    }

    async fn enable(&self) {
        self.1.store(true, Ordering::Relaxed);
    }

    async fn is_enabled(&self) -> bool {
        self.1.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl ResolvableFrameDependency for FlagDependency {
    async fn resolve(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl UnresolvableFrameDependency for FlagDependency {
    async fn unresolve(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl PersistentObject for FlagDependency {
    fn persistence_id() -> &'static str {
        "FlagDependency$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let is_resolved = PersistenceUtils::serialize_field(self.0.load(Ordering::Relaxed))?;
        let is_enabled = PersistenceUtils::serialize_field(self.1.load(Ordering::Relaxed))?;
        Ok(SerializedComponent::new::<Self>(json!({
            "is_resolved": is_resolved,
            "is_enabled": is_enabled
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;
        let is_resolved = PersistenceUtils::deserialize_atomic::<bool>(
            &mut repr,
            "is_resolved",
            "Cannot deserialize the data indicating if the dependency was resolved or not",
        )?;

        let is_enabled = PersistenceUtils::deserialize_atomic::<bool>(
            &mut repr,
            "is_enabled",
            "Cannot deserialize the data indicating if the dependency was enabled or not",
        )?;

        Ok(FlagDependency(
            Arc::new(AtomicBool::new(is_resolved)),
            Arc::new(AtomicBool::new(is_enabled)),
        ))
    }
}
