use chronographer::task::{CronField, TaskScheduleCron};
use std::str::FromStr;

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




