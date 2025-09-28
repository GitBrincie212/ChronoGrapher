use crate::task::ObserverField;
use crate::task::dependency::{
    FrameDependency, ResolvableFrameDependency, UnresolvableFrameDependency,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(unused_imports)]
use crate::task::TaskMetadata;

/// [`MetadataDependencyResolver`] is a trait used for implementing metadata resolvers. By default,
/// functions and closures implement this trait
///
/// # Usage Note(s)
/// While by default the trait [`MetadataDependencyResolver`] is implemented on functions / closures,
/// they cannot persist, as such it is recommended to manually implement this trait yourself
///
/// # Required Method(s)
/// When implementing [`MetadataDependencyResolver`], one has to also implement the method
/// [`MetadataDependencyResolver::is_resolved`] which is where the main logic for
/// the resolver lives in
///
/// # Object Safety
/// [`MetadataDependencyResolver`] is object safe as seen throughout
/// the code for [`MetadataDependency`]
///
/// # See Also
/// - [`MetadataDependency`]
#[async_trait]
pub trait MetadataDependencyResolver<T: Send + Sync>: Send + Sync + 'static {
    /// This is where the main logic for [`MetadataDependencyResolver`] lives in.
    /// The job of this method is to listen to changes made to a field and when they
    /// happen, it checks if the new change should mark the dependency as resolved
    ///
    /// # Argument(s)
    /// As discussed above, [`MetadataDependencyResolver::is_resolve`] accepts one argument,
    /// that being the changed value as ``value``
    ///
    /// # Returns
    /// A boolean indicating whenever or not the changed value satisfies [`MetadataDependencyResolver`],
    /// as such if [`MetadataDependency`] should be resolved or not
    ///
    /// # See Also
    /// - [`MetadataDependency`]
    /// - [`MetadataDependencyResolver`]
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

/// [`MetadataDependency`] monitors closely a [`TaskMetadata`] field (being an [`ObserverField`])
/// and resolves itself depending on the result from a [`MetadataDependencyResolver`]. Upon a change
/// happens, the resolver computes and then its results are cached to be retrieved efficiently
///
/// # Constructor(s)
/// When constructing a [`MetadataDependency`], one can use [`MetadataDependency::new`] where
/// it accepts the field and a [`MetadataDependencyResolver`] to monitor and act accordingly
///
/// # Trait Implementation(s)
/// It is self-explanatory that [`MetadataDependency`] implements [`FrameDependency`], but it
/// also implements [`ResolvableFrameDependency`] and [`UnresolvableFrameDependency`] for manual
/// resolving / unresolving from the developer
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
///
/// # See Also
/// - [`TaskMetadata`]
/// - [`ObserverField`]
/// - [`MetadataDependencyResolver`]
/// - [`FrameDependency`]
/// - [`UnresolvableFrameDependency`]
/// - [`ResolvableFrameDependency`]
/// - [`MetadataDependency::new`]
pub struct MetadataDependency<T: Send + Sync + 'static> {
    field: ObserverField<T>,
    is_resolved: Arc<AtomicBool>,
    resolver: Arc<dyn MetadataDependencyResolver<T>>,
    is_enabled: Arc<AtomicBool>,
}

impl<T: Send + Sync> MetadataDependency<T> {
    /// Creates / Constructs a new [`MetadataDependency`] instance
    ///
    /// # Argument(s)
    /// This method accepts two single arguments, those being ``field``, which is
    /// the field to monitor closely and a [`MetadataDependencyResolver`] as ``resolver``
    /// for acting upon changes
    ///
    /// # Returns
    /// The newly created [`MetadataDependency`] with the field to monitor being ``field``
    /// and the resolving behavior being [`MetadataDependencyResolver`]
    ///
    /// # See Also
    /// - [`MetadataDependencyResolver`]
    /// - [`MetadataDependency`]
    pub fn new(field: ObserverField<T>, resolver: impl MetadataDependencyResolver<T>) -> Self {
        let slf = Self {
            field,
            is_resolved: Arc::new(AtomicBool::new(false)),
            resolver: Arc::new(resolver),
            is_enabled: Arc::new(AtomicBool::new(true)),
        };

        let cloned_resolver = slf.resolver.clone();
        let cloned_is_resolved = slf.is_resolved.clone();

        slf.field.subscribe(move |v| {
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
