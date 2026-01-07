use chronographer::scheduler::timing_wheel_core::TimingWheelCore;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[tokio::test]
async fn peek_ready_prior_to_tick() {
    let interval = Duration::from_millis(25);
    let wheel: TimingWheelCore<u64, &'static str> = TimingWheelCore::new(interval, 8);

    let now = SystemTime::now();
    let id = 100u64;
    wheel.insert(id, Arc::new("peek"), now + interval).await;

    let peek = wheel.peek_ready().await;
    assert!(peek.is_some());
    let (payload, _, got_id) = peek.unwrap();
    assert_eq!(*payload, "peek");
    assert_eq!(got_id, id);
}

#[tokio::test]
async fn insert_and_tick_order() {
    let interval = Duration::from_millis(50);
    let wheel: TimingWheelCore<u64, &'static str> = TimingWheelCore::new(interval, 8);

    let now = SystemTime::now();
    let id1 = 1u64;
    let id2 = 2u64;

    wheel.insert(id1, Arc::new("a"), now + interval).await;
    wheel.insert(id2, Arc::new("b"), now + interval * 3).await;

    let r = wheel.tick().await;
    assert_eq!(r.len(), 1);
    assert_eq!(*r[0].0, "a");

    let r = wheel.tick().await;
    assert!(r.is_empty());

    let r = wheel.tick().await;
    assert_eq!(r.len(), 1);
    assert_eq!(*r[0].0, "b");
}

#[tokio::test]
async fn move_task_later_and_earlier() {
    let interval = Duration::from_millis(20);
    let wheel: TimingWheelCore<u64, &'static str> = TimingWheelCore::new(interval, 10);

    let now = SystemTime::now();
    let id = 7u64;
    wheel.insert(id, Arc::new("x"), now + interval).await;

    wheel
        .move_task(id, now + interval * 4)
        .await
        .expect("move later ok");

    let mut found = false;
    for _ in 0..16 {
        let r = wheel.tick().await;
        if !r.is_empty() {
            assert_eq!(r.len(), 1);
            assert_eq!(*r[0].0, "x");
            found = true;
            break;
        }
    }
    assert!(found, "moved task should become ready within bounded ticks");
}

#[tokio::test]
async fn remove_before_ready_and_clear() {
    let interval = Duration::from_millis(30);
    let wheel: TimingWheelCore<u64, &'static str> = TimingWheelCore::new(interval, 6);
    let now = SystemTime::now();

    let id = 42u64;
    wheel.insert(id, Arc::new("z"), now + interval).await;

    assert!(wheel.exists(&id).await);

    wheel.remove(&id).await;

    assert!(wheel.tick().await.is_empty());
    assert!(!wheel.exists(&id).await);

    let id2 = 43u64;
    wheel.insert(id2, Arc::new("w"), now + interval).await;
    assert!(wheel.exists(&id2).await);

    wheel.clear().await;
    assert!(!wheel.exists(&id2).await);
    assert!(wheel.tick().await.is_empty());
}
