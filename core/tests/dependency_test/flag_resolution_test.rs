use chronographer::prelude::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[tokio::test]
async fn test_flag_resolution() {
    let flag = Arc::new(AtomicBool::new(false));
    let dep = FlagDependency::new(flag.clone());

    assert!(
        !dep.is_resolved().await,
        "Dependency should be unresolved initially"
    );

    dep.resolve().await;
    assert!(
        dep.is_resolved().await,
        "Dependency should be resolved after calling resolve()"
    );

    dep.unresolve().await;
    assert!(
        !dep.is_resolved().await,
        "Dependency should be unresolved after calling unresolve()"
    );

    assert!(
        dep.is_enabled().await,
        "Dependency should be enabled by default"
    );
    dep.disable().await;
    assert!(
        !dep.is_enabled().await,
        "Dependency should be disabled after disable()"
    );
    dep.enable().await;
    assert!(
        dep.is_enabled().await,
        "Dependency should be enabled after enable()"
    );
}