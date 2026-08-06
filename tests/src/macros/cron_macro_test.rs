use chronographer::cron;
use chronographer::task::{CronField, TaskScheduleCron};

#[test]
fn test_every_second() {
    let schedule = cron!(* * * * * *);
    assert_eq!(
        schedule,
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

#[test]
fn test_exact_minute() {
    let schedule = cron!(0 30 * * * *);
    assert_eq!(
        schedule,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(30),
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
        ])
    );
}

#[test]
fn test_exact_hour() {
    let schedule = cron!(0 0 12 * * *);
    assert_eq!(
        schedule,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Exact(0),
            CronField::Exact(12),
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
        ])
    );
}

#[test]
fn test_step() {
    let schedule = cron!(0 0/5 * * * *);
    assert_eq!(
        schedule,
        TaskScheduleCron::new([
            CronField::Exact(0),
            CronField::Step(Box::new(CronField::Exact(0)), 5),
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
            CronField::Wildcard,
        ])
    );
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
