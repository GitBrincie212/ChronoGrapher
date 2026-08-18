use chronographer::task::{
    CronField::{self, Wildcard},
    TaskSchedule, TaskScheduleCron,
};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ts(unix_secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(unix_secs)
}

/// Reference timestamps, all in UTC
const JAN_1_2026: u64 = 1767225600; // Thursday
const JAN_2_2026: u64 = 1767312000; // Friday
const JAN_3_2026: u64 = 1767398400; // Saturday
const JAN_4_2026: u64 = 1767484800; // Sunday
const JAN_5_2026: u64 = 1767571200; // Monday
const JAN_6_2026: u64 = 1767657600; // Tuesday
const JAN_9_2026: u64 = 1767916800; // Friday
const JAN_15_2026: u64 = 1768435200; // Thursday
const JAN_16_2026: u64 = 1768521600; // Friday
const JAN_28_2026: u64 = 1769558400; // Wednesday
const JAN_29_2026: u64 = 1769644800; // Thursday
const JAN_30_2026: u64 = 1769731200; // Friday
const JAN_31_2026: u64 = 1769817600; // Saturday
const FEB_1_2026: u64 = 1769904000; // Sunday
const FEB_2_2026: u64 = 1769990400; // Monday
const FEB_5_2026: u64 = 1770249600; // Thursday
const FEB_15_2026: u64 = 1771113600; // Sunday
const FEB_16_2026: u64 = 1771200000; // Monday
const FEB_27_2026: u64 = 1772150400; // Friday
const FEB_28_2026: u64 = 1772236800; // Saturday
const MAR_1_2026: u64 = 1772323200; // Sunday
const MAR_30_2026: u64 = 1774828800; // Monday
const APR_30_2026: u64 = 1777507200; // Thursday
const JUN_1_2026: u64 = 1780272000; // Monday
const NOV_30_2026: u64 = 1795996800; // Monday
const DEC_1_2026: u64 = 1796083200; // Wednesday
const DEC_31_2026: u64 = 1798675200; // Thursday
const DEC_31_2026_END: u64 = 1798761599; // Dec 31 2026 23:59:59
const JAN_1_2027: u64 = 1798761600; // Friday
const DEC_31_2027: u64 = 1830211200; // Thursday
const FEB_29_2028: u64 = 1835395200; // Tuesday
const JAN_1_2029: u64 = 1861920000; // Monday
const JAN_1_2030: u64 = 1893456000; // Tuesday

const SEC: u64 = 1;
const MIN: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86400;

async fn assert_next(expr: &str, now: u64, expected: u64) {
    let schedule = TaskScheduleCron::from_str(expr).expect("expression should parse");
    let next = schedule.schedule(ts(now)).await.expect("should resolve");
    assert_eq!(next, ts(expected), "for {expr:?} scheduled from {now}");
}

async fn assert_no_next(expr: &str, now: u64) {
    let schedule = TaskScheduleCron::from_str(expr).expect("expression should parse");
    assert!(
        schedule.schedule(ts(now)).await.is_err(),
        "expected {expr:?} from {now} to have no valid scheduling time"
    );
}

#[tokio::test]
async fn every_second() {
    assert_next("* * * * * *", JAN_1_2026, JAN_1_2026 + SEC).await;
    assert_next(
        "* * * * * *",
        JAN_1_2026 + 23 * HOUR + 59 * MIN + 59,
        JAN_2_2026,
    )
    .await;
    assert_next(
        "* * * * * *",
        JAN_31_2026 + 23 * HOUR + 59 * MIN + 59,
        FEB_1_2026,
    )
    .await;
}

#[tokio::test]
async fn exact_second() {
    assert_next("58 * * * * *", JAN_1_2026, JAN_1_2026 + 58).await;
    assert_next("58 * * * * *", JAN_1_2026 + 30, JAN_1_2026 + 58).await;
    assert_next("58 * * * * *", JAN_1_2026 + 58, JAN_1_2026 + MIN + 58).await;
}

#[tokio::test]
async fn second_step() {
    assert_next("*/5 * * * * *", JAN_1_2026, JAN_1_2026 + 5).await;
    assert_next("*/5 * * * * *", JAN_1_2026 + 7, JAN_1_2026 + 10).await;
    assert_next("*/5 * * * * *", JAN_1_2026 + 55, JAN_1_2026 + MIN).await;
}

#[tokio::test]
async fn second_range() {
    assert_next("10-20 * * * * *", JAN_1_2026, JAN_1_2026 + 10).await;
    assert_next("10-20 * * * * *", JAN_1_2026 + 15, JAN_1_2026 + 16).await;
    assert_next("10-20 * * * * *", JAN_1_2026 + 20, JAN_1_2026 + MIN + 10).await;
}

#[tokio::test]
async fn sub_minute_second_rollover() {
    assert_next("0 * * * * *", JAN_1_2026 + 59, JAN_1_2026 + MIN).await;
    assert_next("*/30 * * * * *", JAN_1_2026 + 59, JAN_1_2026 + 60).await;
    assert_next(
        "0 * * * * *",
        JAN_1_2026 + 23 * HOUR + 59 * MIN + 59,
        JAN_2_2026,
    )
    .await;
    assert_next(
        "*/30 * * * * *",
        JAN_1_2026 + 23 * HOUR + 59 * MIN + 59,
        JAN_2_2026,
    )
    .await;
}

#[tokio::test]
async fn exact_minute() {
    assert_next("0 30 * * * *", JAN_1_2026, JAN_1_2026 + 30 * MIN).await;
    assert_next(
        "0 30 * * * *",
        JAN_1_2026 + 30 * MIN,
        JAN_1_2026 + HOUR + 30 * MIN,
    )
    .await;
    assert_next(
        "0 30 * * * *",
        JAN_1_2026 + 23 * HOUR + 59 * MIN + 1,
        JAN_2_2026 + 30 * MIN,
    )
    .await;
}

#[tokio::test]
async fn minute_step() {
    assert_next("0 0/5 * * * *", JAN_1_2026, JAN_1_2026 + 5 * MIN).await;
    assert_next("0 0/30 * * * *", JAN_1_2026, JAN_1_2026 + 30 * MIN).await;
    assert_next("0 0/5 * * * *", JAN_1_2026 + 3 * MIN, JAN_1_2026 + 5 * MIN).await;
    assert_next("0 0/5 * * * *", JAN_1_2026 + 58 * MIN, JAN_1_2026 + HOUR).await;
}

#[tokio::test]
async fn exact_hour() {
    assert_next("0 0 12 * * *", JAN_1_2026, JAN_1_2026 + 12 * HOUR).await;
    assert_next(
        "0 30 10 * * *",
        JAN_1_2026,
        JAN_1_2026 + 10 * HOUR + 30 * MIN,
    )
    .await;
    assert_next(
        "0 0 12 * * *",
        JAN_1_2026 + 12 * HOUR,
        JAN_2_2026 + 12 * HOUR,
    )
    .await;
    assert_next(
        "0 0 12 * * *",
        JAN_1_2026 + 23 * HOUR + 59 * MIN + 59,
        JAN_2_2026 + 12 * HOUR,
    )
    .await;
}

#[tokio::test]
async fn time_of_day_rollover() {
    assert_next("0 0 12 * * ?", JAN_1_2026 + 1, JAN_1_2026 + 12 * HOUR).await;
    assert_next(
        "0 0 12 * * ?",
        JAN_1_2026 + 12 * HOUR + 1,
        JAN_2_2026 + 12 * HOUR,
    )
    .await;
}

#[tokio::test]
async fn next_day_at_midnight() {
    assert_next("0 0 0 * * *", JAN_1_2026, JAN_1_2026 + DAY).await;
    assert_next("0 0 0 * * *", JAN_31_2026, FEB_1_2026).await;
}

#[tokio::test]
async fn exact_day_of_month() {
    assert_next("0 0 0 1 * *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 15 * *", JAN_1_2026, JAN_15_2026).await;
    assert_next("0 0 0 31 * *", JAN_1_2026, JAN_31_2026).await;
    assert_next("0 0 0 15 * *", JAN_15_2026, FEB_1_2026 + 14 * DAY).await;
    assert_next("0 0 0 1 * *", JAN_31_2026, FEB_1_2026).await;
}

#[tokio::test]
async fn day_list() {
    assert_next("0 0 0 1,15 * *", JAN_1_2026, JAN_15_2026).await;
    assert_next("0 0 0 1,15 * *", JAN_15_2026, FEB_1_2026).await;
    assert_next("0 0 0 1,15 * *", JAN_5_2026, JAN_15_2026).await;
}

#[tokio::test]
async fn day_range() {
    assert_next("0 0 0 15-20 * *", JAN_15_2026, JAN_16_2026).await;
    assert_next("0 0 0 1-7 1 *", JAN_1_2026, JAN_2_2026).await;
    assert_next(
        "0 0 0 15-20 * *",
        JAN_1_2026 + 19 * DAY,
        FEB_1_2026 + 14 * DAY,
    )
    .await;
    assert_next("0 0 0 1-7 1 *", JAN_1_2026 + 6 * DAY, JAN_1_2027).await;
}

#[tokio::test]
async fn exact_month() {
    assert_next("0 0 0 1 2 *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 15 2 *", JAN_1_2026, FEB_15_2026).await;
    assert_next("0 0 0 1 12 *", NOV_30_2026, DEC_1_2026).await;
    assert_next("0 0 0 1 3 *", JAN_1_2026, MAR_1_2026).await;
}

#[tokio::test]
async fn month_list() {
    assert_next("0 0 0 * 1,6 *", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 1 1,6 *", JAN_1_2026, JUN_1_2026).await;
    assert_next("0 0 0 * 1,6 *", NOV_30_2026, JAN_1_2027).await;
}

#[tokio::test]
async fn month_and_day_together() {
    assert_next("0 0 0 31 12 *", JAN_1_2026, DEC_31_2026).await;
    assert_next("0 0 0 31 12 *", DEC_31_2026, DEC_31_2027).await;
}

#[tokio::test]
async fn last_second_of_the_year() {
    assert_next("59 59 23 31 12 *", JAN_1_2026, DEC_31_2026_END).await;
    assert_next("* * * * * *", DEC_31_2026_END, JAN_1_2027).await;
}

#[tokio::test]
async fn unspecified_dow_keeps_day_of_month() {
    assert_next("0 0 0 1 * ?", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 1 * ?", JAN_31_2026, FEB_1_2026).await;
}

#[tokio::test]
async fn unspecified_dom_keeps_day_of_week() {
    assert_next("0 0 0 ? * FRI", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 ? * SUN", JAN_1_2026, JAN_4_2026).await;
    assert_next("0 0 0 ? * FRI", JAN_29_2026, JAN_30_2026).await;
}

#[tokio::test]
async fn wildcard_dom_with_weekday() {
    assert_next("0 0 0 * * MON", JAN_1_2026, JAN_5_2026).await;
    assert_next("0 0 0 ? * MON-FRI", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 * * MON", JAN_4_2026, JAN_5_2026).await;
    assert_next("0 0 0 ? * MON-FRI", JAN_4_2026, JAN_5_2026).await;
}

#[tokio::test]
async fn weekday_range() {
    assert_next("0 0 0 ? * 2-6", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 ? * 2-6", JAN_4_2026, JAN_5_2026).await;
}

#[tokio::test]
async fn both_dom_and_dow_specified_use_and() {
    // Jan 1 + Monday: 2026-01-01 is Thursday, 2027 Friday, 2028 Saturday,
    // 2029 Monday, so the first hit is Jan 1 2029.
    assert_next("0 0 0 1 1 MON", JAN_1_2026, JAN_1_2029).await;
    assert_next("0 0 0 1 1 MON", JAN_1_2027, JAN_1_2029).await;
}

#[tokio::test]
async fn month_names() {
    assert_next("0 0 0 1 JAN *", JAN_1_2026, JAN_1_2027).await;
    assert_next("0 0 0 1 FEB *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 1 JUN *", JAN_1_2026, JUN_1_2026).await;
    assert_next("0 0 0 1 DEC *", JAN_1_2026, DEC_1_2026).await;
    assert_next("0 0 0 1 JAN *", DEC_1_2026, JAN_1_2027).await;
}

#[tokio::test]
async fn weekday_names_are_case_insensitive() {
    assert_next("0 0 0 ? * mon", JAN_1_2026, JAN_5_2026).await; // Monday
    assert_next("0 0 0 ? * Fri", JAN_1_2026, JAN_2_2026).await; // Friday
    assert_next("0 0 0 ? * SAT", JAN_1_2026, JAN_3_2026).await; // Saturday
    assert_next("0 0 0 ? * Fri", JAN_2_2026 + 12 * HOUR, JAN_9_2026).await;
}

#[tokio::test]
async fn last_day_of_month() {
    assert_next("0 0 0 L * *", JAN_1_2026, JAN_31_2026).await;
    assert_next("0 0 0 L * *", FEB_1_2026, FEB_28_2026).await;
    assert_next("0 0 0 L * *", FEB_28_2026, MAR_30_2026 + DAY).await;
    assert_next("0 0 0 L * *", APR_30_2026, APR_30_2026 + 31 * DAY).await;
}

#[tokio::test]
async fn last_day_offset() {
    assert_next("0 0 0 L-3 * *", JAN_1_2026, JAN_28_2026).await;
    assert_next("0 0 0 L-3 * *", FEB_1_2026, FEB_1_2026 + 24 * DAY).await;
}

#[tokio::test]
async fn last_day_of_february() {
    assert_next("0 0 0 L 2 *", JAN_1_2026, FEB_28_2026).await;
    assert_next("0 0 0 L 2 *", JAN_1_2027 + 365 * DAY, FEB_29_2028).await;
}

#[tokio::test]
async fn last_day_offset_below_month_start_never_fires() {
    // `L-30` in February targets day -2, which cannot exist; it must not clamp onto day 1.
    assert_no_next("0 0 0 L-30 2 *", JAN_1_2026).await;
}

#[tokio::test]
async fn last_weekday_of_month() {
    // Last Friday of Jan 2026 (Jan 31 is a Saturday).
    assert_next("0 0 0 ? * 6L", JAN_1_2026, JAN_30_2026).await;
    assert_next("0 0 0 ? * 6L", JAN_15_2026, JAN_30_2026).await;
    assert_next(
        "0 15 10 ? * 6L",
        JAN_1_2026,
        JAN_30_2026 + 10 * HOUR + 15 * MIN,
    )
    .await;
    assert_next("0 0 0 ? * 6L", FEB_1_2026, FEB_27_2026).await;
}

#[tokio::test]
async fn bare_last_in_day_of_week() {
    assert_next("0 0 0 ? * L", JAN_1_2026, JAN_31_2026).await;
    assert_next("0 0 0 ? * L", FEB_1_2026, FEB_28_2026).await;
}

#[tokio::test]
async fn nearest_weekday_matches_weekday_itself() {
    assert_next("0 0 0 15W * *", JAN_1_2026, JAN_15_2026).await;
    assert_next("0 0 0 15W * *", FEB_1_2026, FEB_16_2026).await;
}

#[tokio::test]
async fn nearest_weekday_rolls_back_from_saturday() {
    // Jan 10 2026 is a Saturday, so 10W fires on Friday Jan 9.
    assert_next("0 0 0 10W * *", JAN_1_2026, JAN_9_2026).await;
    // Jan 31 2026 is a Saturday, so 31W fires on Friday Jan 30.
    assert_next("0 0 0 31W * *", JAN_1_2026, JAN_30_2026).await;
}

#[tokio::test]
async fn nearest_weekday_rolls_forward_from_sunday() {
    // Feb 1 2026 is a Sunday, so 1W fires on Monday Feb 2.
    assert_next("0 0 0 1W * *", FEB_1_2026, FEB_2_2026).await;
    // Feb 15 2026 is a Sunday, so 15W fires on Monday Feb 16.
    assert_next("0 0 0 15W 2 *", JAN_1_2026, FEB_16_2026).await;
    assert_next("0 0 0 4W * *", JAN_4_2026, JAN_5_2026).await;
}

#[tokio::test]
async fn nearest_weekday_no_match_in_short_month() {
    // `31W` in a month with fewer than 31 days: day 31 doesn't exist, so the job
    // never fires. After scanning 4 years without a match the scheduler bails.
    let schedule = TaskScheduleCron::from_str("0 0 0 31W 2 *").unwrap();
    assert!(schedule.schedule(ts(JAN_1_2026)).await.is_err());

    let schedule = TaskScheduleCron::from_str("0 0 0 31W 4 *").unwrap();
    assert!(schedule.schedule(ts(JAN_1_2026)).await.is_err());

    let schedule = TaskScheduleCron::from_str("0 0 0 31W 11 *").unwrap();
    assert!(schedule.schedule(ts(JAN_1_2026)).await.is_err());
}

#[tokio::test]
async fn last_weekday_of_month_lw() {
    // `LW` = last weekday of the month. Jan 31 2026 is a Saturday, so it is Friday Jan 30.
    assert_next("0 0 0 LW * *", JAN_1_2026, JAN_30_2026).await;
    assert_next("0 0 0 LW * *", FEB_1_2026, FEB_27_2026).await;
}

#[tokio::test]
async fn nth_weekday_first_occurrence() {
    assert_next("0 0 0 ? * 1#1", JAN_1_2026, JAN_4_2026).await; // 1st Sunday
    assert_next("0 0 0 ? * 3#1", JAN_1_2026, JAN_6_2026).await; // 1st Tuesday
    // 1st Thursday of Jan 2026 is Jan 1, which already passed.
    assert_next("0 0 0 ? * 5#1", JAN_1_2026, FEB_5_2026).await;
    assert_next("0 0 0 ? * 1#1", JAN_31_2026, FEB_1_2026).await;
}

#[tokio::test]
async fn nth_weekday_later_occurrences() {
    assert_next("0 0 0 ? * 5#3", JAN_1_2026, JAN_15_2026).await; // 3rd Thursday
    assert_next("0 0 0 ? * 5#5", JAN_1_2026, JAN_29_2026).await; // 5th Thursday
    assert_next("0 0 0 ? * 7#5", JAN_1_2026, JAN_31_2026).await; // 5th Saturday
    assert_next("0 0 0 ? * 2#5", JAN_1_2026, MAR_30_2026).await; // 5th Monday
}

#[tokio::test]
async fn nth_weekday_missing_this_month_rolls_forward() {
    // Feb and Mar 2026 have no 5th Thursday (the 5th one lands on Apr 2), so the
    // schedule must skip to April.
    assert_next("0 0 0 ? * 5#5", FEB_1_2026, APR_30_2026).await;
    assert_next("0 0 0 ? * 5#5", JAN_1_2026, JAN_29_2026).await;
}

#[tokio::test]
async fn feb_29_only_exists_in_leap_years() {
    // 2026 and 2027 are not leap years, so the next Feb 29 is in 2028.
    assert_next("0 0 0 29 2 *", JAN_1_2026, FEB_29_2028).await;
    assert_next("0 0 0 29 2 ?", JAN_1_2026, FEB_29_2028).await;
    assert_next(
        "0 0 0 29 2 *",
        JAN_1_2027 + 365 * DAY + 31 * DAY,
        FEB_29_2028,
    )
    .await;
}

#[tokio::test]
async fn feb_31_never_exists() {
    assert_no_next("0 0 0 31 2 *", JAN_1_2026).await;
}

#[tokio::test]
async fn leap_year_boundary() {
    let schedule = TaskScheduleCron::new(with_year(
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(29),
            CronField::Exact(2),
            CronField::Unspecified,
            CronField::Wildcard,
        ],
        CronField::Exact(2028),
    ));
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));
    assert!(schedule.schedule(ts(FEB_29_2028)).await.is_err());
}

const JAN_1_2026_ARRAY: [CronField; 7] = [
    CronField::Exact(0),
    CronField::Exact(0),
    CronField::Exact(0),
    CronField::Exact(1),
    CronField::Exact(1),
    CronField::Unspecified,
    CronField::Wildcard,
];

#[tokio::test]
async fn wildcard_year_rolls_forward() {
    // Jan 1 with a wildcard year from Jan 1 2026 -> Jan 1 2027.
    let schedule = TaskScheduleCron::new(JAN_1_2026_ARRAY);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2027));
    let next = schedule.schedule(ts(JAN_1_2026 + 12 * HOUR)).await.unwrap();
    assert_eq!(next, ts(JAN_1_2027));
}

#[tokio::test]
async fn exact_year_skips_ahead() {
    let schedule = TaskScheduleCron::new(with_year(JAN_1_2026_ARRAY, CronField::Exact(2027)));
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(JAN_1_2027)
    );

    let schedule = TaskScheduleCron::new(with_year(JAN_1_2026_ARRAY, CronField::Exact(2030)));
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(JAN_1_2030)
    );

    let schedule = TaskScheduleCron::new(with_year(
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(31),
            CronField::Exact(12),
            CronField::Unspecified,
            CronField::Wildcard,
        ],
        CronField::Exact(2027),
    ));
    assert_eq!(
        schedule.schedule(ts(DEC_31_2026)).await.unwrap(),
        ts(DEC_31_2027)
    );
}

#[tokio::test]
async fn exact_year_in_same_year_stays() {
    let schedule = TaskScheduleCron::new(with_year(JAN_1_2026_ARRAY, CronField::Exact(2026)));
    assert!(schedule.schedule(ts(DEC_31_2026)).await.is_err());
    assert!(schedule.schedule(ts(FEB_1_2026)).await.is_err());
}

#[tokio::test]
async fn exact_year_with_month_year_rollover() {
    let schedule = TaskScheduleCron::new(with_year(
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(31),
            CronField::Exact(12),
            CronField::Unspecified,
            CronField::Wildcard,
        ],
        CronField::Exact(2026),
    ));
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(DEC_31_2026)
    );

    let schedule = TaskScheduleCron::new(with_year(
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(31),
            CronField::Exact(12),
            CronField::Unspecified,
            CronField::Wildcard,
        ],
        CronField::Exact(2027),
    ));
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(DEC_31_2027)
    );
}

#[test]
fn new_and_from_str_construct_equal_schedules() {
    let from_str = TaskScheduleCron::from_str("0 0 12 * * ?").unwrap();
    let constructed = TaskScheduleCron::new([
        CronField::Exact(0),
        CronField::Exact(0),
        CronField::Exact(12),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Unspecified,
        CronField::Wildcard,
    ]);
    assert_eq!(from_str, constructed);
}

fn with_wildcard_year(
    [seconds, minute, hour, day_of_month, month, day_of_week]: [CronField; 6],
) -> [CronField; 7] {
    [
        seconds,
        minute,
        hour,
        day_of_month,
        month,
        day_of_week,
        CronField::Wildcard,
    ]
}

fn with_year(mut fields: [CronField; 7], year: CronField) -> [CronField; 7] {
    fields[6] = year;
    fields
}

fn assert_new_matches(expr: &str, expected: [CronField; 6]) {
    let from_str =
        TaskScheduleCron::from_str(expr).unwrap_or_else(|e| panic!("{expr:?} should parse: {e}"));
    let constructed = TaskScheduleCron::new(with_wildcard_year(expected));
    assert_eq!(from_str, constructed, "new/from_str mismatch for {expr:?}");
}

#[test]
fn new_and_from_str_agree_on_basics() {
    assert_new_matches(
        "* * * * * *",
        [Wildcard, Wildcard, Wildcard, Wildcard, Wildcard, Wildcard],
    );
    assert_new_matches(
        "5 0 * * * *",
        [
            CronField::Exact(5),
            CronField::Exact(0),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 12 * * ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(12),
            Wildcard,
            Wildcard,
            CronField::Unspecified,
        ],
    );
    assert_new_matches(
        "0 0 0 1 * ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(1),
            Wildcard,
            CronField::Unspecified,
        ],
    );
    // Maximum valid value in each field.
    assert_new_matches(
        "59 59 23 31 12 *",
        [
            CronField::Exact(59),
            CronField::Exact(59),
            CronField::Exact(23),
            CronField::Exact(31),
            CronField::Exact(12),
            Wildcard,
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_ranges() {
    assert_new_matches(
        "10-20 * * * * *",
        [
            CronField::Range(10, 20),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 15-20 * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Range(15, 20),
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 ? * MON-FRI",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Range(2, 6),
        ],
    );
    assert_new_matches(
        "0 0 0 1 JAN-JUN ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(1),
            CronField::Range(1, 6),
            CronField::Unspecified,
        ],
    );
    assert_new_matches(
        "0 0 0 ? * SUN-SAT",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Range(1, 7),
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_steps() {
    assert_new_matches(
        "*/5 * * * * *",
        [
            CronField::Step(Box::new(Wildcard), 5),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0/30 * * * * *",
        [
            CronField::Step(Box::new(CronField::Exact(0)), 30),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );

    assert_new_matches(
        "5-15/3 * * * * *",
        [
            CronField::Step(Box::new(CronField::Range(5, 15)), 3),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "*/1 * * * * *",
        [
            CronField::Step(Box::new(Wildcard), 1),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_lists() {
    assert_new_matches(
        "1,2,3 * * * * *",
        [
            CronField::List(vec![
                CronField::Exact(1),
                CronField::Exact(2),
                CronField::Exact(3),
            ]),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 1,15 * ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::List(vec![CronField::Exact(1), CronField::Exact(15)]),
            Wildcard,
            CronField::Unspecified,
        ],
    );
    assert_new_matches(
        "0 0 0 ? * MON,WED,FRI",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::List(vec![
                CronField::Exact(2),
                CronField::Exact(4),
                CronField::Exact(6),
            ]),
        ],
    );

    assert_new_matches(
        "0 0 0 1,15-17,*/2 * ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::List(vec![
                CronField::Exact(1),
                CronField::Range(15, 17),
                CronField::Step(Box::new(Wildcard), 2),
            ]),
            Wildcard,
            CronField::Unspecified,
        ],
    );

    assert_new_matches(
        "0 0 0 1,1 * ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::List(vec![CronField::Exact(1), CronField::Exact(1)]),
            Wildcard,
            CronField::Unspecified,
        ],
    );
    assert_new_matches(
        "0 0 0 ? * MON,3",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::List(vec![CronField::Exact(2), CronField::Exact(3)]),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * 1#2,3#4",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::List(vec![
                CronField::NthWeekday(1, 2),
                CronField::NthWeekday(3, 4),
            ]),
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_names() {
    assert_new_matches(
        "0 0 0 ? * SUN",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Exact(1),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * SAT",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Exact(7),
        ],
    );

    assert_new_matches(
        "0 0 0 ? * mon",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Exact(2),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * Fri",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Exact(6),
        ],
    );
    assert_new_matches(
        "0 0 0 1 JAN ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(1),
            CronField::Exact(1),
            CronField::Unspecified,
        ],
    );
    assert_new_matches(
        "0 0 0 1 dec *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(1),
            CronField::Exact(12),
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 1 JUN ?",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(1),
            CronField::Exact(6),
            CronField::Unspecified,
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_dom_operators() {
    assert_new_matches(
        "0 0 0 L * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Last(None),
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 L-3 * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Last(Some(3)),
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 L-1 * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Last(Some(1)),
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 15W * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::NearestWeekday(15),
            Wildcard,
            Wildcard,
        ],
    );
    assert_new_matches(
        "0 0 0 LW * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::NearestWeekday(0),
            Wildcard,
            Wildcard,
        ],
    );

    assert_new_matches(
        "0 0 0 3L * *",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Last(Some(3)),
            Wildcard,
            Wildcard,
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_dow_operators() {
    assert_new_matches(
        "0 0 0 ? * L",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Last(None),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * 6L",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Last(Some(6)),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * L-3",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Last(Some(3)),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * 3L",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::Last(Some(3)),
        ],
    );
    assert_new_matches(
        "0 0 0 ? * 5#3",
        [
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            Wildcard,
            CronField::NthWeekday(5, 3),
        ],
    );
}

#[test]
fn new_and_from_str_agree_on_field_counts() {
    // A five-field expression leaves day-of-week as a wildcard, matching the explicit
    // six-field form.
    let five = TaskScheduleCron::from_str("5 0 * * *").unwrap();
    let six = TaskScheduleCron::from_str("5 0 * * * *").unwrap();
    assert_eq!(five, six);
    assert_new_matches(
        "5 0 * * *",
        [
            CronField::Exact(5),
            CronField::Exact(0),
            Wildcard,
            Wildcard,
            Wildcard,
            Wildcard,
        ],
    );
}

#[test]
fn rejects_empty_expression_without_panicking() {
    assert!(TaskScheduleCron::from_str("").is_err());
}

#[test]
fn rejects_both_day_fields_unspecified() {
    assert!(TaskScheduleCron::from_str("0 0 0 ? * ?").is_err());
}

#[test]
fn rejects_values_out_of_range() {
    for expr in [
        "60 * * * * *", // seconds
        "0 60 * * * *", // minutes
        "0 0 24 * * *", // hours
        "0 0 0 0 * *",  // day of month
        "0 0 0 32 * *", // day of month
        "0 0 0 * 0 *",  // month
        "0 0 0 * 13 *", // month
        "0 0 0 * * 0",  // day of week
        "0 0 0 * * 8",  // day of week
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_invalid_ranges_and_steps() {
    for expr in [
        "5-1 * * * * *", // start > end
        "1/0 * * * * *", // zero step
        "-1 * * * * *",  // leading minus
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_unspecified_in_non_day_fields() {
    // `?` is only meaningful in day_of_month / day_of_week; everywhere else it can never
    // match anything, so it must be rejected rather than silently accepted.
    for expr in [
        "? * * * * *", // seconds
        "0 ? * * * *", // minutes
        "0 0 ? * * *", // hours
        "0 0 0 * ? *", // month
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }

    // A single unspecified day field (the other being wildcard) is still valid.
    assert!(TaskScheduleCron::from_str("0 0 0 ? * *").is_ok());
    assert!(TaskScheduleCron::from_str("0 0 0 * * ?").is_ok());
}

#[test]
fn rejects_out_of_range_step_bases() {
    // The step operator's base is validated like any other value: out-of-range bases
    // must be rejected instead of producing a schedule that silently never fires.
    for expr in [
        "60/2 * * * * *",      // seconds base out of range
        "100-200/2 * * * * *", // range base entirely out of range
        "50-70/2 * * * * *",   // range base partially out of range
        "0 0 24/2 * * *",      // hours base out of range
        "0 0 0 32/2 * *",      // day-of-month base out of range
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_out_of_range_last_offsets() {
    for expr in [
        "0 0 0 L-0 * *",   // day-of-month offset 0
        "0 0 0 L-31 * *",  // day-of-month offset beyond any month length
        "0 0 0 L-100 * *", // day-of-month offset far beyond range
        "0 0 0 ? * 0L",    // day-of-week weekday 0
        "0 0 0 ? * 8L",    // day-of-week weekday 8
        "0 0 0 ? * 100L",  // day-of-week weekday 100
        "0 0 0 ? * L-8",   // day-of-week weekday 8 via L-n syntax
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }

    // Offsets that can still land on a real day remain accepted (an offset that simply
    // falls outside a given month is tolerated, per `last_day_offset_below_month_start_never_fires`).
    assert!(TaskScheduleCron::from_str("0 0 0 L-30 * *").is_ok());
    assert!(TaskScheduleCron::from_str("0 0 0 ? * 6L").is_ok());
}

#[test]
fn rejects_step_over_last() {
    // Stepping over `L` is undefined (it would silently treat `L` as the field's minimum
    // value), so it must be rejected rather than produce a surprising schedule.
    for expr in [
        "0 0 0 L/2 * *",   // step over L in day-of-month
        "0 0 0 L-3/2 * *", // step over an offset L in day-of-month
        "0 0 0 ? * L/2",   // step over L in day-of-week
        "0 0 0 ? * 6L/2",  // step over a weekday L in day-of-week
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_dangling_operators_without_panicking() {
    // A trailing operator leaves the parser past the end of its tokens; this must produce a
    // parse error rather than an index-out-of-bounds panic.
    for expr in [
        "1- * * * * *",   // range missing its end
        "5/ * * * * *",   // step missing its value
        "1# * * * * *",   // nth weekday missing its occurrence
        "L- * * * * *",   // last-of missing its offset
        "15W- * * * * *", // nearest weekday followed by a dangling minus
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_operators_in_wrong_fields() {
    for expr in [
        "0 0 0 L L *",   // L in the month field
        "5W * * * * *",  // W in the seconds field
        "0 0 0 ? * 1#6", // nth beyond 5
        "0 0 0 ? * 0#1", // weekday 0
    ] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_seven_field_expressions() {
    // The string parser only accepts five or six fields; the trailing year is rejected.
    for expr in ["0 0 0 * * * 2027", "0 0 0 31 12 2026", "0 0 0 1 1 * 2030"] {
        assert!(
            TaskScheduleCron::from_str(expr).is_err(),
            "expected {expr:?} to be rejected"
        );
    }
}

#[test]
fn rejects_unknown_names() {
    assert!(TaskScheduleCron::from_str("0 0 0 ? * MONDAY").is_err());
}
