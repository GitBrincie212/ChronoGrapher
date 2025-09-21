use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// [`FlagDependency`] is a dependency which can be enabled and disabled from outside, essentially
/// acting as a checkbox. This dependency can be enabled/disabled and even developers can manually resolve
/// it or manually unresolve it
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
pub struct FlagDependency(Arc<AtomicBool>, Arc<AtomicBool>);

impl FlagDependency {
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
