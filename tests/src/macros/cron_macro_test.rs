use chronographer::cron;
use chronographer::prelude::*;
use chronographer::task::{CronField, TaskScheduleCron};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ts(unix_secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(unix_secs)
}

const JAN_1_2026: u64 = 1767225600;

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
    assert_eq!(next, ts(JAN_1_2026 + 30 * 60));
}

#[tokio::test]
async fn test_exact_hour() {
    let schedule = cron!(0 0 12 * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 12 * 3600));
}

#[tokio::test]
async fn test_step() {
    let schedule = cron!(0 0/5 * * * *);
    let now = ts(JAN_1_2026);
    let next = schedule.schedule(now).await.unwrap();
    assert_eq!(next, ts(JAN_1_2026 + 5 * 60));
}

#[test]
fn test_last_and_nearest_weekday_syntax() {
    let last_weekday = cron!(0 0 0 ? * 6L);
    assert_eq!(
        last_weekday,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Unspecified,
            CronField::Wildcard,
            CronField::Last(Some(6)),
            CronField::Wildcard,
        ])
    );

    let nearest_weekday = cron!(0 0 8 15W * ?);
    assert_eq!(
        nearest_weekday,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(8),
            CronField::NearestWeekday(15),
            CronField::Wildcard,
            CronField::Unspecified,
            CronField::Wildcard,
        ])
    );
}
