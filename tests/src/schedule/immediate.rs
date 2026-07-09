use std::{time::{Duration, SystemTime, UNIX_EPOCH},};

use chronographer::task::{TaskSchedule, TaskScheduleImmediate};

#[tokio::test]
async fn test_schedule_immediate() {
    let instance = TaskScheduleImmediate;
    let now = SystemTime::now();
    let resolve = instance.schedule(now).await.unwrap();

    assert_eq!(resolve, now)
}

#[tokio::test]
async fn test_stateless() {
    let instance = TaskScheduleImmediate;
    let t1 = UNIX_EPOCH;
    let t2 = UNIX_EPOCH + Duration::from_secs(1);
    let resolve = instance.schedule(t1).await.unwrap();
    let resolve2 = instance.schedule(t2).await.unwrap();

    assert_eq!(resolve, t1);
    assert_eq!(resolve2, t2);
}
