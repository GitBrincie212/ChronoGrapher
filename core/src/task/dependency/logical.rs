use crate::task::dependency::FrameDependency;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};

pub enum LogicalDependency {
    AND {
        dep1: Box<dyn FrameDependency>,
        dep2: Box<dyn FrameDependency>,
        is_enabled: AtomicBool,
    },

    OR {
        dep1: Box<dyn FrameDependency>,
        dep2: Box<dyn FrameDependency>,
        is_enabled: AtomicBool,
    },

    XOR {
        dep1: Box<dyn FrameDependency>,
        dep2: Box<dyn FrameDependency>,
        is_enabled: AtomicBool,
    },

    NOT(Box<dyn FrameDependency>, AtomicBool),
}

impl LogicalDependency {
    pub fn and(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::AND {
            dep1: Box::new(dep1),
            dep2: Box::new(dep2),
            is_enabled: AtomicBool::new(true),
        }
    }

    pub fn or(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::OR {
            dep1: Box::new(dep1),
            dep2: Box::new(dep2),
            is_enabled: AtomicBool::new(true),
        }
    }

    pub fn xor(dep1: impl FrameDependency, dep2: impl FrameDependency) -> Self {
        LogicalDependency::XOR {
            dep1: Box::new(dep1),
            dep2: Box::new(dep2),
            is_enabled: AtomicBool::new(true),
        }
    }

    pub fn not(dep: impl FrameDependency) -> Self {
        LogicalDependency::NOT(Box::new(dep), AtomicBool::new(false))
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
