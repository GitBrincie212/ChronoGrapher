use crate::task::{OnTaskEnd, Task, TaskFrame, TaskHook, TaskHookContext, TaskHookEvent};
use async_trait::async_trait;
use std::num::NonZeroU16;
use std::ops::{BitAnd, BitOr, Not};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

type ExternalFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

enum DependencyInner {
    Flag(Arc<AtomicBool>),
    External(ExternalFn),
    LogicalAnd(Box<DependencyInner>, Box<DependencyInner>),
    LogicalOr(Box<DependencyInner>, Box<DependencyInner>),
    LogicalNot(Box<DependencyInner>),
}

impl DependencyInner {
    fn is_resolved(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        match self {
            DependencyInner::Flag(flag) => {
                Box::pin(std::future::ready(flag.load(Ordering::Relaxed)))
            }
            DependencyInner::External(func) => func(),
            DependencyInner::LogicalAnd(dep1, dep2) => {
                Box::pin(async move { dep1.is_resolved().await && dep2.is_resolved().await })
            }
            DependencyInner::LogicalOr(dep1, dep2) => {
                Box::pin(async move { dep1.is_resolved().await || dep2.is_resolved().await })
            }
            DependencyInner::LogicalNot(dep1) => Box::pin(async move { !dep1.is_resolved().await }),
        }
    }
}

pub struct FrameDependency {
    inner: DependencyInner,
    disabled: AtomicBool,
}

macro_rules! impl_monitor_based_dependency {
    (($flag: ident, $countdown: ident, $payload: ident, $task: expr, $value: expr) -> $body: block) => {{
        struct DependencyTaskMonitor(Arc<AtomicBool>, AtomicU16);

        #[async_trait]
        impl TaskHook<OnTaskEnd> for DependencyTaskMonitor {
            async fn on_event(
                &self,
                _ctx: &TaskHookContext,
                payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>,
            ) {
                let $payload = payload;
                let $flag = &self.0;
                let $countdown = &self.1;
                $body
            }
        }

        let flag = Arc::new(AtomicBool::new(false));
        let monitor = DependencyTaskMonitor(flag.clone(), AtomicU16::new($value.get()));
        $task.attach_hook(Arc::new(monitor)).await;

        FrameDependency {
            inner: DependencyInner::Flag(flag),
            disabled: AtomicBool::new(false),
        }
    }};
}

impl FrameDependency {
    pub async fn runs(task: &Task<impl TaskFrame>, value: NonZeroU16) -> FrameDependency {
        impl_monitor_based_dependency!((flag, countdown, _payload, task, value) -> {
            let res = countdown.fetch_sub(1, Ordering::Relaxed) - 1;
            if res == 0 {
                flag.store(true, Ordering::Relaxed);
            }
        })
    }

    pub async fn successful_runs(
        task: &Task<impl TaskFrame>,
        value: NonZeroU16,
    ) -> FrameDependency {
        impl_monitor_based_dependency!((flag, countdown, payload, task, value) -> {
            if payload.is_some() {
                return;
            }

            let res = countdown.fetch_sub(1, Ordering::Relaxed) - 1;
            if res == 0 {
                flag.store(true, Ordering::Relaxed);
            }
        })
    }

    pub async fn failed_runs(task: &Task<impl TaskFrame>, value: NonZeroU16) -> FrameDependency {
        impl_monitor_based_dependency!((flag, countdown, payload, task, value) -> {
            if payload.is_none() {
                return;
            }

            let res = countdown.fetch_sub(1, Ordering::Relaxed) - 1;
            if res == 0 {
                flag.store(true, Ordering::Relaxed);
            }
        })
    }

    pub fn external<F: Future<Output = bool> + Send>(
        value: impl Fn() -> F + Send + Sync + 'static,
    ) -> FrameDependency {
        let value = Arc::new(value);

        FrameDependency {
            inner: DependencyInner::External(Box::new(move || {
                let value = Arc::clone(&value);

                Box::pin(async move { value().await })
            })),

            disabled: AtomicBool::new(false),
        }
    }

    pub fn disable(&self) {
        self.disabled.store(true, Ordering::Relaxed);
    }

    pub fn enable(&self) {
        self.disabled.store(false, Ordering::Relaxed);
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled.load(Ordering::Relaxed)
    }

    pub async fn is_resolved(&self) -> bool {
        if self.is_disabled() {
            return false;
        }

        self.inner.is_resolved().await
    }
}

impl BitAnd for FrameDependency {
    type Output = FrameDependency;

    fn bitand(self, rhs: Self) -> Self::Output {
        FrameDependency {
            inner: DependencyInner::LogicalAnd(Box::new(self.inner), Box::new(rhs.inner)),
            disabled: AtomicBool::new(false),
        }
    }
}

impl BitOr for FrameDependency {
    type Output = FrameDependency;

    fn bitor(self, rhs: Self) -> Self::Output {
        FrameDependency {
            inner: DependencyInner::LogicalOr(Box::new(self.inner), Box::new(rhs.inner)),
            disabled: AtomicBool::new(false),
        }
    }
}

impl Not for FrameDependency {
    type Output = FrameDependency;

    fn not(self) -> Self::Output {
        FrameDependency {
            inner: DependencyInner::LogicalNot(Box::new(self.inner)),
            disabled: AtomicBool::new(false),
        }
    }
}
