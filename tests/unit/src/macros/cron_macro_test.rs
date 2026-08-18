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
const JAN_25_2026: u64 = 1769299200; // Sunday
const JAN_30_2026: u64 = 1769731200; // Friday
const JAN_31_2026: u64 = 1769817600; // Saturday
const FEB_1_2026: u64 = 1769904000; // Sunday
const FEB_2_2026: u64 = 1769990400; // Monday
const FEB_19_2026: u64 = 1771459200; // Thursday
const FEB_25_2026: u64 = 1771977600; // Wednesday
const MAR_15_2026: u64 = 1773532800; // Sunday
const MAR_29_2026: u64 = 1774742400; // Sunday
const MAR_31_2026: u64 = 1774915200; // Tuesday
const FEB_28_2026: u64 = 1772236800; // Saturday
const MAR_1_2026: u64 = 1772323200; // Sunday
const APR_1_2026: u64 = 1775001600; // Wednesday
const APR_30_2026: u64 = 1777507200; // Thursday
const JAN_1_2027: u64 = 1798761600; // Friday
const JAN_25_2028: u64 = 1832371200; // Tuesday
const FEB_1_2028: u64 = 1832976000; // Tuesday
const FEB_24_2028: u64 = 1834963200; // Thursday
const FEB_25_2028: u64 = 1835049600; // Friday
const FEB_29_2028: u64 = 1835395200; // Tuesday

const MIN: u64 = 60;
const HOUR: u64 = 3600;

#[tokio::test]
async fn test_every_second() {
    let schedule = cron!(* * * * * *);

    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 1));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 23 * HOUR + 59 * MIN + 59))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_2_2026));

    let next = schedule
        .schedule(ts(JAN_31_2026 + 23 * HOUR + 59 * MIN + 59))
        .await
        .unwrap();
    assert_eq!(next, ts(FEB_1_2026));

    let next = schedule.schedule(ts(JAN_1_2027 - 1)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2027));
}

#[tokio::test]
async fn test_exact_minute() {
    let schedule = cron!(0 30 * * * *);

    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 30 * MIN));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 30 * MIN - 1))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 30 * MIN));

    let next = schedule.schedule(ts(JAN_1_2026 + 30 * MIN)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + HOUR + 30 * MIN));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 23 * HOUR + 59 * MIN + 59))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_2_2026 + 30 * MIN));
}

#[tokio::test]
async fn test_exact_hour() {
    let schedule = cron!(0 0 12 * * *);

    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 12 * HOUR));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 12 * HOUR - 1))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 12 * HOUR));

    let next = schedule.schedule(ts(JAN_1_2026 + 12 * HOUR)).await.unwrap();
    assert_eq!(next, ts(JAN_2_2026 + 12 * HOUR));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 23 * HOUR + 59 * MIN + 59))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_2_2026 + 12 * HOUR));
}

#[tokio::test]
async fn test_step() {
    let schedule = cron!(0 0/5 * * * *);

    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 5 * MIN));

    let next = schedule.schedule(ts(JAN_1_2026 + 2 * MIN)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 5 * MIN));

    let next = schedule.schedule(ts(JAN_1_2026 + 58 * MIN)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + HOUR));

    let next = schedule
        .schedule(ts(JAN_1_2026 + 23 * HOUR + 58 * MIN))
        .await
        .unwrap();
    assert_eq!(next, ts(JAN_2_2026));
}

#[tokio::test]
async fn test_day_constraints() {
    let schedule = cron!(0 0 0 1 * *);

    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_1_2026));

    let next = schedule.schedule(ts(JAN_31_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_1_2026));

    let next = schedule.schedule(ts(FEB_28_2026)).await.unwrap();
    assert_eq!(next, ts(MAR_1_2026));

    let next = schedule.schedule(ts(APR_30_2026)).await.unwrap();
    assert_eq!(next, ts(APR_30_2026 + 24 * HOUR));
}

#[tokio::test]
async fn test_unspecified_semantics() {
    let schedule = cron!(0 0 0 1 * ?);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_1_2026));

    let schedule = cron!(0 0 0 15 * ?);
    let next = schedule.schedule(ts(FEB_25_2026)).await.unwrap();
    assert_eq!(next, ts(MAR_15_2026));

    let schedule = cron!(0 0 0 29 * ?);
    let next = schedule.schedule(ts(FEB_1_2026)).await.unwrap();
    assert_eq!(next, ts(MAR_29_2026));

    let schedule = cron!(0 0 0 29 * ?);
    let next = schedule.schedule(ts(FEB_1_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));

    let schedule = cron!(0 0 0 ? * 5L);
    let next = schedule.schedule(ts(FEB_1_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_24_2028));
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

    let next = schedule.schedule(ts(FEB_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_28_2026));

    let next = schedule.schedule(ts(FEB_1_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));

    let next = schedule.schedule(ts(APR_1_2026)).await.unwrap();
    assert_eq!(next, ts(APR_30_2026));

    let next = schedule.schedule(ts(JAN_31_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_28_2026));
}

#[tokio::test]
async fn test_nth_weekday() {
    let schedule = cron!(0 0 0 ? * 5#3);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_15_2026));

    let schedule = cron!(0 0 0 ? * 5#3);
    let next = schedule.schedule(ts(JAN_25_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_19_2026));

    let schedule = cron!(0 0 0 ? * 3#5);
    let next = schedule.schedule(ts(FEB_1_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));

    let schedule = cron!(0 0 0 ? * 3#5);
    let next = schedule.schedule(ts(FEB_1_2026)).await.unwrap();
    assert_eq!(next, ts(MAR_31_2026));
}

#[tokio::test]
async fn test_nearest_weekday() {
    let schedule = cron!(0 0 0 15W * *);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_15_2026));

    let schedule = cron!(0 0 0 1W * *);
    let next = schedule.schedule(ts(JAN_25_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_2_2026));

    let schedule = cron!(0 0 0 29W 2 *);
    let next = schedule.schedule(ts(FEB_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));

    let schedule = cron!(0 0 0 29W 2 *);
    let next = schedule.schedule(ts(FEB_1_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));

    let schedule = cron!(0 0 0 25W * *);
    let next = schedule.schedule(ts(JAN_25_2028)).await.unwrap();
    assert_eq!(next, ts(FEB_25_2028));
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
