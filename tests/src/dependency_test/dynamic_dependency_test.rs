use chronographer::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::test]
async fn test_dynamic_dependency() {
    let dep = DynamicDependency::new(|| async { true });
    assert!(
        dep.is_resolved().await,
        "Dynamic dependency should resolve to true based on its future"
    );
    assert!(
        !dep.is_enabled().await,
        "Dynamic dependency should be disabled by default"
    );

    dep.enable().await;
    assert!(
        dep.is_enabled().await,
        "Dynamic dependency should be enabled after calling enable()"
    );

    dep.disable().await;
    assert!(
        !dep.is_enabled().await,
        "Dynamic dependency should be disabled after calling disable()"
    );

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();

    let stateful_dep = DynamicDependency::new(move || {
        let f = flag_clone.clone();
        async move { f.load(Ordering::Relaxed) }
    });

    assert!(
        !stateful_dep.is_resolved().await,
        "Stateful dynamic dependency should initially resolve to false"
    );

    flag.store(true, Ordering::Relaxed);
    assert!(
        stateful_dep.is_resolved().await,
        "Stateful dynamic dependency should resolve to true after underlying state changes"
    );
}