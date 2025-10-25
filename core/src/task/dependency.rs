pub mod dynamic;  // skipcq: RS-D1001

pub mod flag;  // skipcq: RS-D1001

pub mod logical;  // skipcq: RS-D1001

pub mod metadata;  // skipcq: RS-D1001

pub mod task;  // skipcq: RS-D1001

pub use dynamic::*;
pub use flag::*;
pub use logical::*;
pub use metadata::*;
use std::ops::Deref;
pub use task::*;

use async_trait::async_trait;

#[allow(unused_imports)]
use crate::task::DependencyTaskFrame;

/// [`FrameDependency`] describes a dependency for [`DependencyTaskFrame`] which have to be
/// resolved in order to proceed. Dependencies can wrap other dependencies, creating a hierarchy
/// (otherwise known as a **Dependency Tree**) that allows for flexibility on how a developer
/// defines their dependencies
///
/// # Required Method(s)
/// When developing [`FrameDependency`] the main method to consider implementing is
/// [`FrameDependency::is_resolved`], which handles the logic for determining if a
/// dependency is resolved
///
/// **Pro Tip:** Try to use caching whenever possible to prevent expensive dependency
/// recomputation when the task frame is executed
///
/// There are other required methods as well, which include [`FrameDependency::disable`] and
/// [`FrameDependency::enable`] for handling enabling/disabling dependencies (effectively skipping
/// them) and [`FrameDependency::is_enabled`] for checking if a dependency is enabled
///
/// # Trait Implementation(s)
/// Currently, ChronoGrapher offers 5 such implementations of this trait:
/// 1. [`DynamicDependency`] It is a function which execute every time it is called
/// 2. [`FlagDependency`] A flag that can be toggled on/off from outside parties to resolve/unresolve it
/// 3. [`LogicalDependency`] Wraps other dependencies in boolean operations (AND, OR, NOT & XOR)
/// 4. [`MetadataDependency`] Tracks closely a field in the metadata and automatically resolves
/// when it is updated to a correct value defined by a resolver
/// 5. [`TaskDependency`] Monitors a task closely, watching how many runs has it completed (with
/// errors, successes or both) and determines if the dependency is resolved based on if the
/// run count has surpassed the maximum threshold
///
/// # Extension Trait(s)
/// Currently [`FrameDependency`] has 2 hooks traits for managing manual resolving and
/// unresolving of a [`FrameDependency`] those being [`ResolvableFrameDependency`] and
/// [`UnresolvableFrameDependency`] respectively. The former is for automatic resolving of
/// a dependency while the latter is effectively resetting the frame dependency to not resolved
///
/// The reason these are 2 separate traits is due to the fact, some dependencies can be
/// manually resolved but not unresolved or the opposite. They are also hooks traits because
/// some dependencies may not allow manual resolve or manual unresolve, instead, handling it themselves
///
/// # Object Safety
/// This trait is object-safe as seen in the source code of [`DependencyTaskFrame`]
///
/// # See Also
/// - [`DynamicDependency`]
/// - [`FlagDependency`]
/// - [`LogicalDependency`]
/// - [`MetadataDependency`]
/// - [`DependencyTaskFrame`]
/// - [`UnresolvableFrameDependency`]
/// - [`ResolvableFrameDependency`]
#[async_trait]
pub trait FrameDependency: Send + Sync + 'static {
    /// Checks if the dependency is resolved or not, one can
    /// still execute this method even if the dependency is
    /// disabled via [`FrameDependency::disable`]
    ///
    /// # Returns
    /// A boolean value indicating if the dependency has been resolved
    /// or not, true if it has, false if it hasn't still been resolved
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`FrameDependency::disable`]
    async fn is_resolved(&self) -> bool;

    /// Disables the dependency, blocking [`FrameDependency`] from calling it, this
    /// can be useful when an outside parties doesn't want to care about the dependency
    /// or even for optimization on non-cachable uncontrollable dependencies. If of
    /// course one ensures the dependency will always be resolved during the disable downtime
    ///
    /// If the dependency has already been disabled, it effectively is a no-op,
    /// the opposite of this method is [`FrameDependency::enable`]. One can also
    /// view if the dependency is enabled or not via [`FrameDependency::is_enabled`]
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`FrameDependency::enable`]
    /// - [`FrameDependency::is_enabled`]
    async fn disable(&self);

    /// Enables the dependency, allowing [`FrameDependency`] from calling it, this
    /// can be useful when an outside parties at some point want to account the dependency
    /// or even for optimization on non-cachable dependencies (via disabling and enabling
    /// when needed)
    ///
    /// If the dependency has already been enabled, it effectively is a no-op,
    /// The opposite of this method is [`FrameDependency::disable`]. One can also
    /// view if the dependency is enabled or not via [`FrameDependency::is_enabled`]
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`FrameDependency::disable`]
    /// - [`FrameDependency::is_enabled`]
    async fn enable(&self);

    /// Checks if the dependency is enabled, one can manipulate this state
    /// via [`FrameDependency::enable`] for enabling the dependency and oppositely
    /// [`FrameDependency::disable`]. This method is used internally by [`DependencyTaskFrame`]
    ///
    /// # Returns
    /// A boolean value indicating if the dependency is resolved or not
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`FrameDependency::enable`]
    /// - [`FrameDependency::disable`]
    async fn is_enabled(&self) -> bool;
}

#[async_trait]
impl<D> FrameDependency for D
where
    D: Deref + ?Sized + Send + Sync + 'static,
    D::Target: FrameDependency,
{
    async fn is_resolved(&self) -> bool {
        self.deref().is_resolved().await
    }

    async fn disable(&self) {
        self.deref().disable().await
    }

    async fn enable(&self) {
        self.deref().enable().await
    }

    async fn is_enabled(&self) -> bool {
        self.deref().is_enabled().await
    }
}

/// [`ResolvableFrameDependency`] Represents a resolvable [`FrameDependency`], this dependency
/// can be automatically resolved, or it may be manually resolved. The opposite of this trait
/// is [`UnresolvableFrameDependency`]
///
/// # Required Method(s)
/// If one plans to implement [`ResolvableFrameDependency`], then they need to implement
/// [`ResolvableFrameDependency::resolve`] which manually resolves the dependency
///
/// # Supertrait(s)
/// For implementing this trait, one has to also implement the [`FrameDependency`] since
/// this is an hooks of a dependency to allow resolving it manually
///
/// # Trait Implementation(s)
/// Specifically, there are 3 dependencies which implement [`ResolvableFrameDependency`], those being:
/// - [`MetadataDependency`]
/// - [`TaskDependency`]
/// - [`FlagDependency`]
///
/// # See Also
/// - [`FrameDependency`]
/// - [`MetadataDependency`]
/// - [`TaskDependency`]
/// - [`FlagDependency`]
#[async_trait]
pub trait ResolvableFrameDependency: FrameDependency {
    /// Resolves manually a [`FrameDependency`]
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`ResolvableFrameDependency`]
    async fn resolve(&self);
}

/// [`UnresolvableFrameDependency`] Represents an unresolvable [`FrameDependency`], this dependency
/// can be manually unresolved, essentially resetting its state. The opposite of this trait
/// is [`ResolvableFrameDependency`]
///
/// # Required Method(s)
/// If one plans to implement [`UnresolvableFrameDependency`], then they need to implement
/// [`UnresolvableFrameDependency::unresolve`] which manually unresolves the dependency
///
/// # Supertrait(s)
/// For implementing this trait, one has to also implement the [`FrameDependency`] since
/// this is an hooks of a dependency to allow unresolving it manually
///
/// # Trait Implementation(s)
/// Specifically, there are 3 dependencies which implement [`UnresolvableFrameDependency`], those being:
/// - [`MetadataDependency`]
/// - [`TaskDependency`]
/// - [`FlagDependency`]
///
/// # See Also
/// - [`FrameDependency`]
/// - [`MetadataDependency`]
/// - [`TaskDependency`]
/// - [`FlagDependency`]
#[async_trait]
pub trait UnresolvableFrameDependency: FrameDependency {
    /// Unresolves manually a [`FrameDependency`], effectively
    /// resetting its state
    ///
    /// # See Also
    /// - [`FrameDependency`]
    /// - [`ResolvableFrameDependency`]
    async fn unresolve(&self);
}
