use chronographer::task::{CronField, TaskSchedule, TaskScheduleCron};
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
}

#[tokio::test]
async fn exact_second() {
    assert_next("58 * * * * *", JAN_1_2026, JAN_1_2026 + 58).await;
}

#[tokio::test]
async fn second_step() {
    assert_next("*/5 * * * * *", JAN_1_2026, JAN_1_2026 + 5).await;
}

#[tokio::test]
async fn second_range() {
    assert_next("10-20 * * * * *", JAN_1_2026, JAN_1_2026 + 10).await;
}

#[tokio::test]
async fn sub_minute_second_rollover() {
    // At second 59 the next schedule for `0` seconds is the next full minute.
    assert_next("0 * * * * *", JAN_1_2026 + 59, JAN_1_2026 + MIN).await;
    // Similarly, stepping every 30 seconds from second 59 lands at +60s.
    assert_next("*/30 * * * * *", JAN_1_2026 + 59, JAN_1_2026 + 60).await;
}

#[tokio::test]
async fn exact_minute() {
    assert_next("0 30 * * * *", JAN_1_2026, JAN_1_2026 + 30 * MIN).await;
}

#[tokio::test]
async fn minute_step() {
    assert_next("0 0/5 * * * *", JAN_1_2026, JAN_1_2026 + 5 * MIN).await;
    assert_next("0 0/30 * * * *", JAN_1_2026, JAN_1_2026 + 30 * MIN).await;
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
}

#[tokio::test]
async fn time_of_day_rollover() {
    // 12:00:00 from just past midnight stays on the same day.
    assert_next("0 0 12 * * ?", JAN_1_2026 + 1, JAN_1_2026 + 12 * HOUR).await;
}

#[tokio::test]
async fn next_day_at_midnight() {
    assert_next("0 0 0 * * *", JAN_1_2026, JAN_1_2026 + DAY).await;
}

#[tokio::test]
async fn exact_day_of_month() {
    assert_next("0 0 0 1 * *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 15 * *", JAN_1_2026, JAN_15_2026).await;
    assert_next("0 0 0 31 * *", JAN_1_2026, JAN_31_2026).await;
}

#[tokio::test]
async fn day_list() {
    assert_next("0 0 0 1,15 * *", JAN_1_2026, JAN_15_2026).await;
}

#[tokio::test]
async fn day_range() {
    // 15-20 from the 15th rolls to the 16th.
    assert_next("0 0 0 15-20 * *", JAN_15_2026, JAN_16_2026).await;
    assert_next("0 0 0 1-7 1 *", JAN_1_2026, JAN_2_2026).await;
}

#[tokio::test]
async fn exact_month() {
    assert_next("0 0 0 1 2 *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 15 2 *", JAN_1_2026, FEB_15_2026).await;
}

#[tokio::test]
async fn month_list() {
    // Jan is the current month, so the day constraint resolves first.
    assert_next("0 0 0 * 1,6 *", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 1 1,6 *", JAN_1_2026, JUN_1_2026).await;
}

#[tokio::test]
async fn month_and_day_together() {
    assert_next("0 0 0 31 12 *", JAN_1_2026, DEC_31_2026).await;
}

#[tokio::test]
async fn last_second_of_the_year() {
    assert_next("59 59 23 31 12 *", JAN_1_2026, DEC_31_2026_END).await;
    // Rolls across the year boundary.
    assert_next("* * * * * *", DEC_31_2026_END, JAN_1_2027).await;
}

#[tokio::test]
async fn unspecified_dow_keeps_day_of_month() {
    // `?` in the day-of-week field must NOT disable the day-of-month constraint.
    assert_next("0 0 0 1 * ?", JAN_1_2026, FEB_1_2026).await;
}

#[tokio::test]
async fn unspecified_dom_keeps_day_of_week() {
    // `?` in the day-of-month field must NOT disable the day-of-week constraint.
    assert_next("0 0 0 ? * FRI", JAN_1_2026, JAN_2_2026).await;
    assert_next("0 0 0 ? * SUN", JAN_1_2026, JAN_4_2026).await;
}

#[tokio::test]
async fn wildcard_dom_with_weekday() {
    assert_next("0 0 0 * * MON", JAN_1_2026, JAN_5_2026).await;
    assert_next("0 0 0 ? * MON-FRI", JAN_1_2026, JAN_2_2026).await;
}

#[tokio::test]
async fn weekday_range() {
    assert_next("0 0 0 ? * 2-6", JAN_1_2026, JAN_2_2026).await;
}

#[tokio::test]
async fn both_dom_and_dow_specified_use_and() {
    // Jan 1 + Monday: 2026-01-01 is Thursday, 2027 Friday, 2028 Saturday,
    // 2029 Monday, so the first hit is Jan 1 2029.
    assert_next("0 0 0 1 1 MON", JAN_1_2026, JAN_1_2029).await;
}

#[tokio::test]
async fn month_names() {
    assert_next("0 0 0 1 JAN *", JAN_1_2026, JAN_1_2027).await;
    assert_next("0 0 0 1 FEB *", JAN_1_2026, FEB_1_2026).await;
    assert_next("0 0 0 1 JUN *", JAN_1_2026, JUN_1_2026).await;
    assert_next("0 0 0 1 DEC *", JAN_1_2026, DEC_1_2026).await;
}

#[tokio::test]
async fn weekday_names_are_case_insensitive() {
    assert_next("0 0 0 ? * mon", JAN_1_2026, JAN_5_2026).await; // Monday
    assert_next("0 0 0 ? * Fri", JAN_1_2026, JAN_2_2026).await; // Friday
    assert_next("0 0 0 ? * SAT", JAN_1_2026, JAN_3_2026).await; // Saturday
}

#[tokio::test]
async fn last_day_of_month() {
    assert_next("0 0 0 L * *", JAN_1_2026, JAN_31_2026).await;
}

#[tokio::test]
async fn last_day_offset() {
    assert_next("0 0 0 L-3 * *", JAN_1_2026, JAN_28_2026).await;
}

#[tokio::test]
async fn last_day_of_february() {
    assert_next("0 0 0 L 2 *", JAN_1_2026, FEB_28_2026).await;
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
}

#[tokio::test]
async fn bare_last_in_day_of_week() {
    assert_next("0 0 0 ? * L", JAN_1_2026, JAN_31_2026).await;
}

#[tokio::test]
async fn nearest_weekday_matches_weekday_itself() {
    assert_next("0 0 0 15W * *", JAN_1_2026, JAN_15_2026).await;
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
}

#[tokio::test]
async fn nearest_weekday_clamps_to_short_month() {
    // A requested day beyond the month length targets the last day: `31W` in February 2026
    // means the 28th (Saturday), whose nearest weekday is Friday Feb 27.
    assert_next("0 0 0 31W 2 *", JAN_1_2026, FEB_27_2026).await;
    // `31W` in a 30-day month where the 30th is a weekday fires on the 30th.
    assert_next("0 0 0 31W 4 *", JAN_1_2026, APR_30_2026).await;
    assert_next("0 0 0 31W 11 *", JAN_1_2026, NOV_30_2026).await;
}

#[tokio::test]
async fn last_weekday_of_month_lw() {
    // `LW` = last weekday of the month. Jan 31 2026 is a Saturday, so it is Friday Jan 30.
    assert_next("0 0 0 LW * *", JAN_1_2026, JAN_30_2026).await;
}

#[tokio::test]
async fn nth_weekday_first_occurrence() {
    assert_next("0 0 0 ? * 1#1", JAN_1_2026, JAN_4_2026).await; // 1st Sunday
    assert_next("0 0 0 ? * 3#1", JAN_1_2026, JAN_6_2026).await; // 1st Tuesday
    // 1st Thursday of Jan 2026 is Jan 1, which already passed.
    assert_next("0 0 0 ? * 5#1", JAN_1_2026, FEB_5_2026).await;
}

#[tokio::test]
async fn nth_weekday_later_occurrences() {
    assert_next("0 0 0 ? * 5#3", JAN_1_2026, JAN_15_2026).await; // 3rd Thursday
    assert_next("0 0 0 ? * 5#5", JAN_1_2026, JAN_29_2026).await; // 5th Thursday
    assert_next("0 0 0 ? * 7#5", JAN_1_2026, JAN_31_2026).await; // 5th Saturday
    assert_next("0 0 0 ? * 2#5", JAN_1_2026, MAR_30_2026).await; // 5th Monday
}

#[tokio::test]
async fn feb_29_only_exists_in_leap_years() {
    // 2026 and 2027 are not leap years, so the next Feb 29 is in 2028.
    assert_next("0 0 0 29 2 *", JAN_1_2026, FEB_29_2028).await;
    assert_next("0 0 0 29 2 ?", JAN_1_2026, FEB_29_2028).await;
}

#[tokio::test]
async fn feb_31_never_exists() {
    assert_no_next("0 0 0 31 2 *", JAN_1_2026).await;
}

#[tokio::test]
async fn leap_year_boundary() {
    // Feb 29 2028 in year 2028 explicitly.
    let schedule = TaskScheduleCron::new([
        CronField::Exact(0),
        CronField::Exact(0),
        CronField::Exact(0),
        CronField::Exact(29),
        CronField::Exact(2),
        CronField::Unspecified,
        CronField::Exact(2028),
    ]);
    let next = schedule.schedule(ts(JAN_1_2026)).await.unwrap();
    assert_eq!(next, ts(FEB_29_2028));
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
}

#[tokio::test]
async fn exact_year_skips_ahead() {
    let mut fields = JAN_1_2026_ARRAY;
    fields[6] = CronField::Exact(2027);
    let schedule = TaskScheduleCron::new(fields);
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(JAN_1_2027)
    );

    let mut fields = JAN_1_2026_ARRAY;
    fields[6] = CronField::Exact(2030);
    let schedule = TaskScheduleCron::new(fields);
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(JAN_1_2030)
    );
}

#[tokio::test]
async fn exact_year_in_same_year_stays() {
    // Jan 1 2026 from Dec 31 2026 has already passed.
    let mut fields = JAN_1_2026_ARRAY;
    fields[6] = CronField::Exact(2026);
    let schedule = TaskScheduleCron::new(fields);
    assert!(schedule.schedule(ts(DEC_31_2026)).await.is_err());
}

#[tokio::test]
async fn exact_year_with_month_year_rollover() {
    let mut fields = JAN_1_2026_ARRAY;
    fields[3] = CronField::Exact(31);
    fields[4] = CronField::Exact(12);
    fields[6] = CronField::Exact(2026);
    let schedule = TaskScheduleCron::new(fields);
    assert_eq!(
        schedule.schedule(ts(JAN_1_2026)).await.unwrap(),
        ts(DEC_31_2026)
    );

    let mut fields = JAN_1_2026_ARRAY;
    fields[3] = CronField::Exact(31);
    fields[4] = CronField::Exact(12);
    fields[6] = CronField::Exact(2027);
    let schedule = TaskScheduleCron::new(fields);
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

#[test]
fn from_str_defaults_trailing_fields_to_wildcard() {
    // A five-field expression leaves day_of_week as a wildcard.
    let five = TaskScheduleCron::from_str("5 0 * * *").unwrap();
    let six = TaskScheduleCron::from_str("5 0 * * * *").unwrap();
    assert_eq!(five, six);
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
