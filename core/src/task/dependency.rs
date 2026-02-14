pub mod dynamic; // skipcq: RS-D1001
pub mod flag; // skipcq: RS-D1001
pub mod logical; // skipcq: RS-D1001
pub mod task; // skipcq: RS-D1001

pub use dynamic::*;
pub use flag::*;
pub use logical::*;
pub use task::*;

use async_trait::async_trait;
use std::ops::Deref;

#[allow(unused_imports)]
use crate::task::DependencyTaskFrame;

#[async_trait]
pub trait FrameDependency: Send + Sync + 'static {
    async fn is_resolved(&self) -> bool;

    async fn disable(&self);

    async fn enable(&self);

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

#[async_trait]
pub trait ResolvableFrameDependency: FrameDependency {
    async fn resolve(&self);
}

#[async_trait]
pub trait UnresolvableFrameDependency: FrameDependency {
    async fn unresolve(&self);
}
