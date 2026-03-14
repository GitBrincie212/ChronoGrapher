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
        assert_eq!(duration, $expected);
    };
}

const MINUTE: Duration = Duration::from_secs(60);
const HOUR: Duration = Duration::from_secs(3600);
const DAY: Duration = Duration::from_secs(86400);

#[test]
fn test_seconds() {
    assert_every!(Duration::from_secs(2), 2s);
    assert_every!(Duration::from_secs_f64(2.5), 2.5s);
}

#[test]
fn test_millis() {
    assert_every!(Duration::from_millis(1), 1ms);
}

#[test]
fn test_minutes() {
    assert_every!(MINUTE * 3, 3m);
}

#[test]
fn test_hours() {
    assert_every!(HOUR * 4, 4h);
}

#[test]
fn test_days() {
    assert_every!(DAY * 5, 5d);
    assert_every!(Duration::from_secs_f64(5.12 * 86400.0), 5.12d);
}

#[test]
fn test_sep() {
    assert_every!(
        MINUTE * 3 + Duration::from_secs(2) + Duration::from_millis(1),
        3m,
        2s,
        1ms
    );
    assert_every!(DAY * 5 + Duration::from_secs(2), 5d, 2s);
}

#[test]
fn test_spaced() {
    assert_every!(MINUTE * 3 + Duration::from_secs(2) + Duration::from_millis(1), 3m 2s 1ms);
    assert_every!(DAY * 5 + Duration::from_secs(2), 5d 2s);
}

#[test]
fn test_macro_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("ui/every_errors.rs");
}
