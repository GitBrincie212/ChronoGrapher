use chronographer::cron;
use chronographer::prelude::*;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ts(unix_secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(unix_secs)
}

const JAN_1_2026: u64 = 1767225600; // Thursday
const JAN_2_2026: u64 = 1767312000; // Friday
const JAN_15_2026: u64 = 1768435200; // Thursday
const JAN_30_2026: u64 = 1769731200; // Friday
const JAN_31_2026: u64 = 1769817600; // Saturday
const FEB_1_2026: u64 = 1769904000; // Sunday

const MIN: u64 = 60;
const HOUR: u64 = 3600;

#[tokio::test]
async fn test_every_second() {
    let schedule = cron!(* * * * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 1));
}

#[tokio::test]
async fn test_exact_minute() {
    let schedule = cron!(0 30 * * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 30 * MIN));
}

#[tokio::test]
async fn test_exact_hour() {
    let schedule = cron!(0 0 12 * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 12 * HOUR));
}

#[tokio::test]
async fn test_step() {
    let schedule = cron!(0 0/5 * * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 5 * MIN));
}

#[tokio::test]
async fn test_day_constraints() {
    let schedule = cron!(0 0 0 1 * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(FEB_1_2026));
}

#[tokio::test]
async fn test_unspecified_semantics() {
    let schedule = cron!(0 0 0 1 * ?);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_1_2026));

    let schedule = cron!(0 0 0 ? * MON-FRI);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_2_2026));
}

#[tokio::test]
async fn test_weekday_names() {
    let schedule = cron!(0 0 0 ? * MON-FRI);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_2_2026));

    let schedule = cron!(0 15 10 ? * 6L);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_30_2026 + 10 * HOUR + 15 * MIN));
}

#[tokio::test]
async fn test_last_day_of_month() {
    let schedule = cron!(0 0 0 L * *);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_31_2026));
}

#[tokio::test]
async fn test_nth_weekday() {
    let schedule = cron!(0 0 0 ? * 5#3);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_15_2026));
}

#[tokio::test]
async fn test_nearest_weekday() {
    let schedule = cron!(0 0 0 15W * *);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_15_2026));
}

#[tokio::test]
async fn test_macro_matches_from_str() {
    for expr in [
        "* * * * * *",
        "0 0 12 * * ?",
        "0 0 0 ? * MON-FRI",
        "0 15 10 ? * 6L",
        "0 0 0 L-3 * *",
        "0 0 0 LW * *",
    ] {
        let from_str = TaskScheduleCron::from_str(expr).unwrap();
        let next_from_str = from_str.schedule(ts(JAN_1_2026)).await.unwrap();

        let next_macro = match expr {
            "* * * * * *" => cron!(* * * * * *).schedule(ts(JAN_1_2026)).await.unwrap(),
            "0 0 12 * * ?" => cron!(0 0 12 * * ?).schedule(ts(JAN_1_2026)).await.unwrap(),
            "0 0 0 ? * MON-FRI" => cron!(0 0 0 ? * MON-FRI)
                .schedule(ts(JAN_1_2026))
                .await
                .unwrap(),
            "0 15 10 ? * 6L" => cron!(0 15 10 ? * 6L)
                .schedule(ts(JAN_1_2026))
                .await
                .unwrap(),
            "0 0 0 L-3 * *" => cron!(0 0 0 L-3 * *).schedule(ts(JAN_1_2026)).await.unwrap(),
            "0 0 0 LW * *" => cron!(0 0 0 LW * *).schedule(ts(JAN_1_2026)).await.unwrap(),
            _ => unreachable!(),
        };

        assert_eq!(
            next_macro, next_from_str,
            "macro/from_str mismatch for {expr:?}"
        );
    }
}

#[test]
fn test_macro_structure() {
    let t = trybuild::TestCases::new();
    t.compile_fail("ui/cron_errors.rs");
}
