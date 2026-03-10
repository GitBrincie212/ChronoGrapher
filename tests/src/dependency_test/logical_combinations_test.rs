use chronographer::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::test]
async fn test_and_dependency() {
    let f1 = Arc::new(AtomicBool::new(false));
    let f2 = Arc::new(AtomicBool::new(false));

    let d1 = FlagDependency::new(f1.clone());
    let d2 = FlagDependency::new(f2.clone());

    let and_dep = LogicalDependency::and(d1, d2);
    assert!(
        !and_dep.is_resolved().await,
        "AND dependency should not be resolved initially"
    );

    f1.store(true, Ordering::Relaxed);
    assert!(
        !and_dep.is_resolved().await,
        "AND dependency should remain unresolved when only one input is resolved"
    );

    f2.store(true, Ordering::Relaxed);
    assert!(
        and_dep.is_resolved().await,
        "AND dependency should be resolved when both inputs are resolved"
    );

    f1.store(false, Ordering::Relaxed);
    assert!(
        !and_dep.is_resolved().await,
        "AND dependency should remain unresolved when only one input is resolved"
    );
}

#[tokio::test]
async fn test_or_dependency() {
    let f1 = Arc::new(AtomicBool::new(false));
    let f2 = Arc::new(AtomicBool::new(false));

    let d1 = FlagDependency::new(f1.clone());
    let d2 = FlagDependency::new(f2.clone());

    let or_dep = LogicalDependency::or(d1, d2);
    assert!(
        !or_dep.is_resolved().await,
        "OR dependency should not be resolved initially"
    );

    f1.store(true, Ordering::Relaxed);
    assert!(
        or_dep.is_resolved().await,
        "OR dependency should be resolved when at least one input is resolved"
    );

    f2.store(true, Ordering::Relaxed);
    assert!(
        or_dep.is_resolved().await,
        "OR dependency should be resolved when both inputs are resolved"
    );

    f1.store(false, Ordering::Relaxed);
    assert!(
        or_dep.is_resolved().await,
        "OR dependency should be resolved when at least one input is resolved"
    )
}

#[tokio::test]
async fn test_xor_dependency() {
    let f1 = Arc::new(AtomicBool::new(false));
    let f2 = Arc::new(AtomicBool::new(false));

    let d1 = FlagDependency::new(f1.clone());
    let d2 = FlagDependency::new(f2.clone());

    let or_dep = LogicalDependency::xor(d1, d2);
    assert!(
        !or_dep.is_resolved().await,
        "XOR dependency should not be resolved initially"
    );

    f1.store(true, Ordering::Relaxed);
    assert!(
        or_dep.is_resolved().await,
        "XOR dependency should be resolved when at least one input is resolved"
    );

    f2.store(true, Ordering::Relaxed);
    assert!(
        !or_dep.is_resolved().await,
        "XOR dependency should not be resolved when both inputs are resolved"
    );

    f1.store(false, Ordering::Relaxed);
    assert!(
        or_dep.is_resolved().await,
        "XOR dependency should be resolved when at least one input is resolved"
    )
}

#[tokio::test]
async fn test_not_dependency() {
    let f1 = Arc::new(AtomicBool::new(false));

    let d1 = FlagDependency::new(f1.clone());
    f1.store(true, Ordering::Relaxed);
    
    let not_dep = LogicalDependency::not(d1);
    assert!(
        !not_dep.is_resolved().await,
        "NOT dependency should not be resolved if its input is resolved"
    );

    f1.store(false, Ordering::Relaxed);
    assert!(
        not_dep.is_resolved().await,
        "NOT dependency should be resolved if its input is not resolved"
    );
}