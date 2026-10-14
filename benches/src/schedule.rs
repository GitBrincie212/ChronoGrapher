use crate::util::runtime;
use chronographer::task::{
    CronField, TaskSchedule, TaskScheduleCron, TaskScheduleImmediate, TaskScheduleInterval,
};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn fixed_now() -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(1_700_000_000_000)
}

const CRON_EXPRESSIONS: &[&str] = &[
    "* * * * * ?",
    "*/15 * * * * ?",
    "0 * * * * ?",
    "0 */5 * * * ?",
    "0 0 * * * ?",
    "0 0 12 * * ?",
    "0 0 9-17 * * ?",
    "0 0 9-17 * * 1-5",
    "0 15,45 * * * ?",
    "0 30 8 * * 1",
    "0 0 1 1 * ?",
];

#[divan::bench]
fn schedule_immediate(bencher: divan::Bencher) {
    let schedule = TaskScheduleImmediate;
    let now = fixed_now();
    bencher.bench(|| {
        let next = runtime()
            .block_on(schedule.schedule(divan::black_box(now)))
            .unwrap();
        divan::black_box(next);
    });
}

#[divan::bench(args = [10u64, 100, 1_000, 10_000])]
fn schedule_immediate_batch(bencher: divan::Bencher, count: u64) {
    let schedule = TaskScheduleImmediate;
    let now = fixed_now();
    bencher.counter(count).bench(|| {
        let mut next = now;
        runtime().block_on(async {
            for _ in 0..count {
                next = schedule.schedule(divan::black_box(next)).await.unwrap();
            }
        });
        divan::black_box(next);
    });
}

#[divan::bench(args = [Duration::from_millis(1), Duration::from_secs(1), Duration::from_secs(60), Duration::from_secs(3600)])]
fn schedule_interval(bencher: divan::Bencher, interval: Duration) {
    let schedule = TaskScheduleInterval::duration(interval);
    let now = fixed_now();
    bencher.bench(|| {
        let next = runtime()
            .block_on(schedule.schedule(divan::black_box(now)))
            .unwrap();
        divan::black_box(next);
    });
}

#[divan::bench]
fn duration_interval(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(TaskScheduleInterval::duration(Duration::from_secs(1)));
    });
}

#[divan::bench]
fn from_secs_interval(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(TaskScheduleInterval::from_secs(1));
    });
}

#[divan::bench(args = CRON_EXPRESSIONS)]
fn parse_cron(bencher: divan::Bencher, expr: &'static str) {
    bencher.bench(|| {
        let cron = TaskScheduleCron::from_str(divan::black_box(expr)).unwrap();
        divan::black_box(cron);
    });
}

#[divan::bench]
fn new_cron(bencher: divan::Bencher) {
    bencher.bench(|| {
        let fields = std::array::from_fn(|_| CronField::Wildcard);
        divan::black_box(TaskScheduleCron::new(fields));
    });
}

#[divan::bench(args = CRON_EXPRESSIONS)]
fn schedule_cron(bencher: divan::Bencher, expr: &'static str) {
    let cron = TaskScheduleCron::from_str(expr).unwrap();
    let now = fixed_now();
    bencher.bench(|| {
        let next = runtime()
            .block_on(cron.schedule(divan::black_box(now)))
            .unwrap();
        divan::black_box(next);
    });
}

#[divan::bench(args = [10u64, 100, 1_000])]
fn schedule_cron_batch(bencher: divan::Bencher, count: u64) {
    let cron = TaskScheduleCron::from_str("0 0 9-17 * * ?").unwrap();
    let now = fixed_now();
    bencher.counter(count).bench(|| {
        let mut next = now;
        runtime().block_on(async {
            for _ in 0..count {
                next = cron.schedule(divan::black_box(next)).await.unwrap();
            }
        });
        divan::black_box(next);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_expressions_are_valid() {
        for expr in CRON_EXPRESSIONS {
            assert!(
                TaskScheduleCron::from_str(expr).is_ok(),
                "invalid CRON expression: {expr}"
            );
        }
    }

    #[test]
    fn schedule_crons_are_in_the_future() {
        let cron = TaskScheduleCron::from_str("0 0 9-17 * * ?").unwrap();
        let now = fixed_now();
        let next = runtime().block_on(cron.schedule(now)).unwrap();
        assert!(next > now);
    }
}
