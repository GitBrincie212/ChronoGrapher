use crate::task::dependency::FrameDependency;
use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

type DynamicFunction = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

/// [`DynamicDependency`] represents a dependency which hosts a function inside that computes
/// if the dependency is resolved (unlike most dependencies which use a caching mechanism), this
/// dependency type cannot persist disk due to the nature of functions. For those cases, it is
/// advised to create a struct yourself and implement the trait [`FrameDependency`]
///
/// Due to the nature of being a function with no caching mechanism, it cannot be manually
/// resolved or unresolved (tho it can still be disabled)
///
/// As such, while it is provided in the core, this shouldn't be used apart
/// from debugging, experimenting and other niche cases
///
/// # Example
/// ```ignore
/// use chronographer_core::task::dependency::DynamicDependency;
///
/// let dependency = DynamicDependency::new(|_| async {
///     println!("Bip boop, i compute your value!");
///     true // This is just an example
/// });
///
/// // You can then attach it to a DependencyTaskFrame
/// ```
pub struct DynamicDependency {
    func: DynamicFunction,
    is_enabled: Arc<AtomicBool>,
}

impl DynamicDependency {
    pub fn new<Fut, Func>(func: Func) -> Self
    where
        Fut: Future<Output = bool> + Send + 'static,
        Func: Fn() -> Fut + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(move || Box::pin(func()) as Pin<Box<dyn Future<Output = bool> + Send>>),
            is_enabled: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl FrameDependency for DynamicDependency {
    async fn is_resolved(&self) -> bool {
        (self.func)().await
    }

    async fn disable(&self) {
        self.is_enabled.store(false, Ordering::Relaxed);
    }

    async fn enable(&self) {
        self.is_enabled.store(true, Ordering::Relaxed);
    }

    async fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Relaxed)
    }
}
