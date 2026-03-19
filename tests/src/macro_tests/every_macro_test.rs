#![allow(unused)]
// A function passed in test is still seen as unused.
use chronographer::prelude::*;
use std::time::{Duration, SystemTime};

fn simulate_duration(interval: TaskScheduleInterval) -> Duration {
    let now = SystemTime::now();
    let next = interval.schedule(now).unwrap();

    next.duration_since(now).unwrap()
}

macro_rules! assert_every {
    ($expected:expr, $($args:tt)*) => {
        let interval = every!($($args)*);
        let duration = simulate_duration(interval);
        assert_eq!(duration.as_secs_f64(), $expected as f64);
    };
}

const MINUTE: f64 = 60.0;
const HOUR: f64 = 3600.0;
const DAY: f64 = 86400.0;

#[test]
fn test_seconds() {
    assert_every!(2.0, 2s);
    assert_every!(2.5, 2.5s);
    assert_every!(59.999, 59.999s);
}

#[test]
fn test_millis() {
    assert_every!(0.001, 1ms);
    assert_every!(0.000001, 0.001ms);
    assert_every!(0.999999, 999.999ms);
}

#[test]
fn test_minutes() {
    assert_every!(MINUTE * 3.0, 3m);
    assert_every!(MINUTE * 59.999, 59.999m);
}

#[test]
fn test_hours() {
    assert_every!(HOUR * 4.0, 4h);
    assert_every!(HOUR * 59.999, 59.999h);
}

#[test]
fn test_days() {
    assert_every!(DAY * 5.0, 5d);
    assert_every!(5.12 * 86400.0, 5.12d);
    assert_every!(DAY * 31.0, 31d);
}

#[test]
fn test_sep() {
    assert_every!(MINUTE * 3.0 + 2.0 + 0.001, 3m, 2s, 1ms);
    assert_every!(DAY * 5.0 + 2.0, 5d, 2s);
    assert_every!(
        DAY * 1.0 + HOUR * 1.0 + MINUTE * 1.0 + 1.0 + 0.001,
        1d,
        1h,
        1m,
        1s,
        1ms
    );
    assert_every!(
        DAY * 31.0 + HOUR * 59.0 + MINUTE * 59.0 + 59.0 + 0.9999,
        31d,
        59h,
        59m,
        59s,
        999.9ms
    );
    assert_every!(DAY * 1.0 + 0.001, 1d, 1ms);
    assert_every!(HOUR * 1.0 + 1.0, 1h, 1s);
}

#[test]
fn test_spaced() {
    assert_every!(MINUTE * 3.0 + 2.0 + 0.001, 3m 2s 1ms);
    assert_every!(DAY * 5.0 + 2.0, 5d 2s);
    assert_every!(DAY * 1.0 + HOUR * 1.0 + MINUTE * 1.0 + 1.0 + 0.001, 1d 1h 1m 1s 1ms);
}

#[test]
fn test_mixed_floats() {
    assert_every!(DAY * 1.0 + HOUR * 1.0 + MINUTE * 1.5, 1d, 1h, 1.5m);
    assert_every!(DAY * 1.0 + HOUR * 2.5, 1d, 2.5h);
    assert_every!(DAY * 30.5, 30.5d);
}

#[test]
fn test_macro_structure() {
    let t = trybuild::TestCases::new();
    t.compile_fail("ui/every_errors.rs");
}
