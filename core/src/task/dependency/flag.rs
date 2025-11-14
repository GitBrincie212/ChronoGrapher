use crate::persistence::PersistenceObject;
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

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
/// for developers to manually resolve and unresolve. Additionally, serde's [`Serialize`], [`Deserialize`]
/// traits and [`PersistenceObject`]
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
#[derive(Serialize, Deserialize)]
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
impl PersistenceObject for FlagDependency {
    const PERSISTENCE_ID: &'static str = "chronographer::FlagDependency#8e932fba-afec-40c6-b73d-1c048f382ab8";
}
