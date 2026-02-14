use crate::task::dependency::FrameDependency;
use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

type DynamicFunction = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

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
