use chronographer::task::{CronField, TaskSchedule, TaskScheduleCron};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const JAN_1_2026: u64 = 1767225600;

fn ts(unix_secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(unix_secs)
}

#[tokio::test]
async fn test_parse_every_second() {
    let task_cron = TaskScheduleCron::from_str("* * * * * *").unwrap();

    assert_eq!(
        task_cron,
        TaskScheduleCron::new([
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
        ])
    );
}

#[tokio::test]
async fn test_parse_exact_daily() {
    let task_cron = TaskScheduleCron::from_str("0 0 12 * * ?").unwrap();

    assert_eq!(
        task_cron,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(12),
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Unspecified,
            CronField::Wildcard
        ])
    )
}


#[tokio::test]
async fn test_parse_step_field() {
    let task_cron = TaskScheduleCron::from_str("0 0/5 14 * * ?").unwrap();

    assert_eq!(
        task_cron,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Step(Box::new(CronField::Exact(0)), 5),
            CronField::Exact(14),
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Unspecified,
            CronField::Wildcard
        ])
    )
}

#[tokio::test]
async fn test_parse_range_field() {
    let task_cron = TaskScheduleCron::from_str("0 15 10 ? * MON-FRI").unwrap();

    assert_eq!(
        task_cron,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(15),
            CronField::Exact(10),
            CronField::Unspecified,
            CronField::Wildcard,
            CronField::Range(2, 6),
            CronField::Wildcard
        ])
    )
}

#[tokio::test]
async fn test_parse_last_of_month() {
    let task_cron = TaskScheduleCron::from_str("0 15 10 ? * 6L").unwrap();

    assert_eq!(
        task_cron,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(15),
            CronField::Exact(10),
            CronField::Unspecified,
            CronField::Wildcard,
            CronField::Last(Some(6)),
            CronField::Wildcard
        ])
    )
}

#[tokio::test]
async fn test_value_out_of_range() {
    let err = TaskScheduleCron::from_str("0 0 99 * * ?").unwrap_err();

    assert_eq!(err.field_pos, 2);
    let msg = err.to_string();
    assert!(msg.contains("99"));
    assert!(msg.contains("out of range"));
    assert!(msg.contains("hours"));
    assert!(msg.contains("0-23"));
}

#[tokio::test]
async fn test_invalid_range_start_gt_end() {
    let err = TaskScheduleCron::from_str("0 0 12 * * MON-SUN").unwrap_err();

    assert_eq!(err.field_pos, 5);
    let msg = err.to_string();
    assert!(msg.contains("Invalid range"));
    assert!(msg.contains("day_of_week"));
}

#[tokio::test]
async fn test_invalid_step_zero() {
    let err = TaskScheduleCron::from_str("0 0/0 * * * ?").unwrap_err();

    assert_eq!(err.field_pos, 1);
    let msg = err.to_string();
    assert!(msg.contains("Step value 0 must be greater than 0"));
}

#[tokio::test]
async fn test_last_operator_wrong_field() {
    let err = TaskScheduleCron::from_str("L 0 12 * * ?").unwrap_err();

    assert_eq!(err.field_pos, 0);
    let msg = err.to_string();
    assert!(msg.contains("L (last) operator is only valid for day_of_month and day_of_week fields"));
}

#[tokio::test]
async fn test_nearest_weekday_wrong_field() {
    let err = TaskScheduleCron::from_str("0 15W 12 * * ?").unwrap_err();

    assert_eq!(err.field_pos, 1);
    let msg = err.to_string();
    assert!(msg.contains("W (nearest weekday) operator is only valid for day_of_month field"));
}

#[tokio::test]
async fn test_nth_weekday_out_of_bounds() {
    let err = TaskScheduleCron::from_str("0 0 12 ? * MON#6").unwrap_err();

    assert_eq!(err.field_pos, 5);
    let msg = err.to_string();
    assert!(msg.contains("Nth weekday 6 is out of range (expected 1-5)"));
}

#[tokio::test]
async fn test_dom_and_dow_both_unspecified() {
    let err = TaskScheduleCron::from_str("0 0 12 ? * ?").unwrap_err();

    assert_eq!(err.field_pos, 3);
    let msg = err.to_string();
    assert!(msg.contains("cannot both be unspecified"));
}

#[tokio::test]
async fn test_wildcard_ticks_one_second() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 1));
}

#[tokio::test]
async fn test_exact_field_same_minute() {
    let task_cron = TaskScheduleCron::new([
        CronField::Exact(30),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 30));
}

#[tokio::test]
async fn test_exact_field_rolls_to_next_minute() {
    let task_cron = TaskScheduleCron::new([
        CronField::Exact(30),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026 + 45);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 90));
}

#[tokio::test]
async fn test_hour_rollover_resets_minute_second() {
    let task_cron = TaskScheduleCron::new([
        CronField::Exact(0),
        CronField::Exact(0),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026 + 14 * 3600 + 30 * 60 + 15);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 15 * 3600));
}

#[tokio::test]
async fn test_range_field_matches_within_bounds() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Range(9, 17),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026 + 3 * 3600);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 9 * 3600));
}

#[tokio::test]
async fn test_range_field_rolls_to_next_day() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Range(9, 17),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026 + 18 * 3600);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, ts(JAN_1_2026 + 24 * 3600 + 9 * 3600));
}

#[tokio::test]
async fn test_step_field() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Step(Box::new(CronField::Exact(0)), 15),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026 + 7 * 60);
    let next = task_cron.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 15 * 60));

    let now = ts(JAN_1_2026 + 50 * 60);
    let next = task_cron.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 3600));
}

#[tokio::test]
async fn test_dom_and_dow_both_specified_is_and() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Exact(15),
        CronField::Wildcard,
        CronField::Exact(2),
        CronField::Wildcard,
    ]);

    const MAY_1_2026: u64 = 1777593600;
    let now = ts(MAY_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    const JUN_15_2026: u64 = 1781481600;
    assert_eq!(next, ts(JUN_15_2026));
}

#[tokio::test]
async fn test_dom_specified_dow_unspecified_is_just_dom() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Exact(15),
        CronField::Wildcard,
        CronField::Unspecified,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    const JAN_15_2026: u64 = 1768435200;
    assert_eq!(next, ts(JAN_15_2026));
}

#[tokio::test]
async fn test_month_rollover() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Exact(31),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    const APR_1_2026: u64 = 1775001600;
    let now = ts(APR_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    const MAY_31_2026: u64 = 1780185600;
    assert_eq!(next, ts(MAY_31_2026));
}

#[tokio::test]
async fn test_year_boundary_exhausted() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Exact(2026),
    ]);

    const JAN_1_2027: u64 = 1798761600;
    let now = ts(JAN_1_2027);

    assert!(task_cron.schedule(now).await.is_err());
}

#[tokio::test]
async fn test_leap_year_feb29() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Exact(29),
        CronField::Exact(2),
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026);
    let next = task_cron.schedule(now).await.unwrap();

    const FEB_29_2028: u64 = 1835395200;
    assert_eq!(next, ts(FEB_29_2028));
}

#[tokio::test]
async fn test_last_of_month_matcher() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Last(None),
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026);

    let result = tokio::time::timeout(Duration::from_secs(5), task_cron.schedule(now)).await;

    if let Ok(inner) = result {
        assert!(inner.is_err());
    }
}

#[tokio::test]
async fn test_schedule_ignores_now_subsecond() {
    let task_cron = TaskScheduleCron::new([
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
        CronField::Wildcard,
    ]);

    let now = ts(JAN_1_2026) + Duration::from_millis(500);
    let next = task_cron.schedule(now).await.unwrap();

    assert_eq!(next, now + Duration::from_secs(1));
}

