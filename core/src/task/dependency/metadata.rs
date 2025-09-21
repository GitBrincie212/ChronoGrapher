use crate::task::ObserverField;
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A trait for implementing metadata resolvers. By default, functions and closures implement this
/// trait, but they cannot persist due to the limitations of closures, in those cases it is recommended
/// to create a struct yourself that implements this trait
#[async_trait]
pub trait MetadataDependencyResolver<T: Send + Sync>: Send + Sync + 'static {
    async fn is_resolved(&self, value: Arc<T>) -> bool;
}

#[async_trait]
impl<T, F, Fut> MetadataDependencyResolver<T> for F
where
    T: Send + Sync + 'static,
    F: Fn(Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = bool> + Send + 'static,
{
    async fn is_resolved(&self, value: Arc<T>) -> bool {
        self(value).await
    }
}

/// [`MetadataDependency`] monitors closely a metadata field and resolves itself depending on the
/// result from a supplied resolver function. Upon a change happens, the resolver computes and then
/// its results are cached to be retrieved efficiently
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::task::{DefaultTaskMetadata, TaskMetadata};
/// use chronographer_core::task::dependency::{FrameDependency, MetadataDependency};
///
/// // You won't need to create a task metadata container 99% of the time
/// let metadata = DefaultTaskMetadata::new();
///
/// let mut observed_debug_label = metadata.debug_label();
///
/// let dependency = MetadataDependency::new(observed_debug_label.clone(), |v: Arc<String>| async move {
///     let magic_password = Arc::new("Magic Password".to_owned());
///     let follows_magic_password = v.clone() == magic_password;
///     if follows_magic_password {
///         println!("You guessed the magic password");
///     } else {
///         println!("Not correct, try again")
///     }
///
///     follows_magic_password
/// });
///
/// // ... Some time passes ...
/// observed_debug_label.update(String::from("Hello World"));
/// assert_eq!(dependency.is_resolved().await, false); // Somewhere else
///
/// // ... More time passes ...
/// observed_debug_label.update(String::from("Magic Password"));
/// assert_eq!(dependency.is_resolved().await, true); // Somewhere else
/// ```
pub struct MetadataDependency<T: Send + Sync + 'static> {
    field: ObserverField<T>,
    is_resolved: Arc<AtomicBool>,
    resolver: Arc<dyn MetadataDependencyResolver<T>>,
    is_enabled: Arc<AtomicBool>,
}

impl<T: Send + Sync> MetadataDependency<T> {
    pub fn new(field: ObserverField<T>, resolver: impl MetadataDependencyResolver<T>) -> Self {
        let slf = Self {
            field,
            is_resolved: Arc::new(AtomicBool::new(false)),
            resolver: Arc::new(resolver),
            is_enabled: Arc::new(AtomicBool::new(true)),
        };

        let cloned_resolver = slf.resolver.clone();
        let cloned_is_resolved = slf.is_resolved.clone();

        slf.field.subscribe(move |v: Arc<T>| {
            let cloned_is_resolved = cloned_is_resolved.clone();
            let cloned_resolver = cloned_resolver.clone();
            async move {
                let resolved = cloned_resolver.is_resolved(v).await;
                cloned_is_resolved.store(resolved, Ordering::Relaxed);
            }
        });

        slf
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FrameDependency for MetadataDependency<T> {
    async fn is_resolved(&self) -> bool {
        self.is_resolved.load(Ordering::Relaxed)
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

#[async_trait]
impl<T: Send + Sync + 'static> ResolvableFrameDependency for MetadataDependency<T> {
    async fn resolve(&self) {
        self.is_resolved.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> UnresolvableFrameDependency for MetadataDependency<T> {
    async fn unresolve(&self) {
        self.is_resolved.store(false, Ordering::Relaxed);
    }
}
