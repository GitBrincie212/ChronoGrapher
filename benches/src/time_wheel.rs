use std::time::Duration;

use chronographer::utils::timing_wheel::HierarchicalTimingWheel;

fn main() {
    divan::main();
}

#[divan::bench]
fn insert_level1() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    for i in 0..100 {
        wheel.insert(i, Duration::from_millis(i));
    }
}

#[divan::bench]
fn insert_level2() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    for i in 0..100 {
        wheel.insert(i, Duration::from_millis(256 + i));
    }
}

#[divan::bench]
fn insert_mixed_levels() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    let delays = [1, 100, 300, 1000, 70_000, 20_000_000];
    for (i, &delay) in delays.iter().enumerate().cycle().take(100) {
        wheel.insert(i as u64, Duration::from_millis(delay));
    }
}

#[divan::bench]
fn tick_empty() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    for _ in 0..256 {
        divan::black_box(wheel.tick());
    }
}

#[divan::bench]
fn tick_with_entries() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    for i in 0..256u64 {
        wheel.insert(i, Duration::from_millis(i));
    }
    for _ in 0..256 {
        divan::black_box(wheel.tick());
    }
}

#[divan::bench]
fn insert_and_clear() {
    let mut wheel = HierarchicalTimingWheel::<u64>::default();
    for i in 0..1000 {
        wheel.insert(i, Duration::from_millis(i));
    }
    wheel.clear();
}
