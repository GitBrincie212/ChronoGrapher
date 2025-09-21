use crate::task::dependency::FrameDependency;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// [`LogicalDependency`] is one of the few non-atomic dependencies, it essentially wraps one
/// or multiple task dependencies to boolean operations. When computed, it computes both dependencies
/// and then based on the operator and the results, it returns the appropriate value.
///
/// [`LogicalDependency`] cannot be manually resolved or manually unresolved due to its nature of
/// being an operator. However, one can manually resolve/unresolve the dependencies it hosts to simulate
/// this kind of behavior. Just like every dependency, it can also be enabled/disabled
///
/// The boolean operations include:
/// - [`LogicalDependency::AND`] simulates the `val1 && val2` (AND operator) for task frame dependencies
/// - [`LogicalDependency::OR`] simulates the `val1 || val2` (OR operator) for task frame dependencies
/// - [`LogicalDependency::XOR`] simulates the `val1 ^ val2` (XOR operator) for task frame dependencies
/// - [`LogicalDependency::NOT`] simulates the `!val` (NOT operator) for task framee dependencies
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::task::dependency::{DynamicDependency, LogicalDependency};
///
/// let logical = LogicalDependency::and(
///     LogicalDependency::not(
///         // First Dependency
///     ),
///
///     // Second Dependency
/// );
///
/// // This only executes if the first dependency isn't resolved and the second is
/// ```
pub enum LogicalDependency {
    AND {
        dep1: Arc<dyn FrameDependency>,
        dep2: Arc<dyn FrameDependency>,
        is_enabled: Arc<AtomicBool>,
    },

    OR {
        dep1: Arc<dyn FrameDependency>,
        dep2: Arc<dyn FrameDependency>,
        is_enabled: Arc<AtomicBool>,
    },

    XOR {
        dep1: Arc<dyn FrameDependency>,
        dep2: Arc<dyn FrameDependency>,
        is_enabled: Arc<AtomicBool>,
    },

    NOT(Arc<dyn FrameDependency>, Arc<AtomicBool>),
}

impl LogicalDependency {
    pub fn and(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::AND {
            dep1: Arc::new(dep1),
            dep2: Arc::new(dep2),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn or(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::OR {
            dep1: Arc::new(dep1),
            dep2: Arc::new(dep2),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn xor(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::XOR {
            dep1: Arc::new(dep1),
            dep2: Arc::new(dep2),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn not(dep: impl FrameDependency) -> Self {
        LogicalDependency::NOT(Arc::new(dep), Arc::new(AtomicBool::new(false)))
    }
}

macro_rules! implement_toggle_functionality {
    ($self: expr, $value: expr) => {
        match $self {
            LogicalDependency::AND { is_enabled, .. } => {
                is_enabled.store($value, Ordering::Relaxed);
            }

            LogicalDependency::XOR { is_enabled, .. } => {
                is_enabled.store($value, Ordering::Relaxed);
            }

            LogicalDependency::OR { is_enabled, .. } => {
                is_enabled.store($value, Ordering::Relaxed);
            }

            LogicalDependency::NOT(_, is_enabled) => {
                is_enabled.store($value, Ordering::Relaxed);
            }
        }
    };
}

#[async_trait]
impl FrameDependency for LogicalDependency {
    async fn is_resolved(&self) -> bool {
        match self {
            LogicalDependency::AND { dep1, dep2, .. } => {
                dep1.is_resolved().await && dep2.is_resolved().await
            }

            LogicalDependency::XOR { dep1, dep2, .. } => {
                dep1.is_resolved().await ^ dep2.is_resolved().await
            }

            LogicalDependency::OR { dep1, dep2, .. } => {
                dep1.is_resolved().await || dep2.is_resolved().await
            }

            LogicalDependency::NOT(dep, _) => !dep.is_resolved().await,
        }
    }

    async fn disable(&self) {
        implement_toggle_functionality!(self, false);
    }

    async fn enable(&self) {
        implement_toggle_functionality!(self, true);
    }

    async fn is_enabled(&self) -> bool {
        match self {
            LogicalDependency::AND { is_enabled, .. } => is_enabled.load(Ordering::Relaxed),

            LogicalDependency::XOR { is_enabled, .. } => is_enabled.load(Ordering::Relaxed),

            LogicalDependency::OR { is_enabled, .. } => is_enabled.load(Ordering::Relaxed),

            LogicalDependency::NOT(_, is_enabled) => is_enabled.load(Ordering::Relaxed),
        }
    }
}
