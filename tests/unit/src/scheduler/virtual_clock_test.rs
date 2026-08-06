use chronographer::scheduler::clock::{AdvanceableSchedulerClock, SchedulerClock, VirtualClock};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tokio::try_join;

// A small value to avoid floating precision errors
const EPSILON: Duration = Duration::from_millis(1);

macro_rules! assert_approx {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        let epsilon_val = EPSILON;
        let diff = if left_val > right_val {
            left_val.duration_since(right_val).unwrap()
        } else {
            right_val.duration_since(left_val).unwrap()
        };
        assert!(
            diff <= epsilon_val,
            "assertion failed: `(left ≈ right)` \
             (left: `{:?}`, right: `{:?}`, difference: `{:?}`, epsilon: `{:?}`)",
            left_val,
            right_val,
            diff,
            epsilon_val
        );
    }};
}

#[tokio::test]
async fn test_initial_epoch() {
    let clock = VirtualClock::from_epoch();
    assert_approx!(clock.now(), UNIX_EPOCH);
}

#[tokio::test]
async fn test_custom_time() {
    let time0 = UNIX_EPOCH + Duration::from_secs(45);
    let clock = VirtualClock::new(time0);
    assert_approx!(clock.now(), time0);
}

#[tokio::test]
async fn test_advance() {
    let clock = VirtualClock::from_epoch();
    clock.advance(Duration::from_secs(1));
    assert_eq!(clock.now(), UNIX_EPOCH + Duration::from_secs(1));
    clock.advance(Duration::from_secs(100));
    assert_eq!(clock.now(), UNIX_EPOCH + Duration::from_secs(101));
}

#[tokio::test]
async fn test_advance_to() {
    let clock = VirtualClock::from_epoch();
    let target = UNIX_EPOCH + Duration::from_secs(19);
    clock.advance_to(target);
    assert_approx!(clock.now(), target);

    let target = UNIX_EPOCH + Duration::from_secs(235);
    clock.advance_to(target);
    assert_approx!(clock.now(), target);
}

#[tokio::test]
async fn test_idle_to_same_time() {
    let clock = VirtualClock::from_epoch();
    clock.advance(Duration::from_secs(5));

    let target = UNIX_EPOCH + Duration::from_secs(5);
    clock.idle_to(target).await;

    let now = clock.now();
    assert_approx!(now, target);
}

#[tokio::test]
async fn test_single_idle_to_past_time() {
    let clock = VirtualClock::from_epoch();
    clock.advance(Duration::from_secs(1));

    clock.idle_to(UNIX_EPOCH).await;
    let now = clock.now();
    assert_approx!(now, UNIX_EPOCH + Duration::from_secs(1));
}

#[tokio::test]
async fn test_single_idle_to_future_time() {
    let clock = Arc::new(VirtualClock::from_epoch());
    clock.advance(Duration::from_secs(1));
    let clock_clone = clock.clone();

    let task1 = tokio::spawn(async move {
        clock_clone
            .idle_to(UNIX_EPOCH + Duration::from_secs(1))
            .await;
        let now = clock_clone.now();
        assert_approx!(now, UNIX_EPOCH + Duration::from_secs(1));
    });

    let task2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        clock.advance(Duration::from_secs(1));
    });

    try_join!(task1, task2).unwrap();
}

#[tokio::test]
async fn test_single_cancelled_idle_to() {
    let clock = Arc::new(VirtualClock::from_epoch());
    let clock_clone = clock.clone();

    let task_idled = Arc::new(AtomicBool::new(false));
    let task_idled_clone = task_idled.clone();
    let task = tokio::spawn(async move {
        clock_clone
            .idle_to(UNIX_EPOCH + Duration::from_secs(1))
            .await;
        task_idled_clone.store(true, Ordering::Release);
    });

    task.abort();
    assert!(
        !task_idled.load(Ordering::Acquire),
        "Task shouldn't have stopped idling due to cancellation"
    );

    clock.advance(Duration::from_secs(1));
    assert!(
        !task_idled.load(Ordering::Acquire),
        "Task shouldn't have stopped idling due to cancellation"
    );
}

fn spawn_idled_task(
    clock: &Arc<VirtualClock>,
    offset: Duration,
) -> (JoinHandle<()>, tokio::sync::oneshot::Receiver<()>) {
    let clock_clone = clock.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();

    let task = tokio::spawn(async move {
        clock_clone.idle_to(UNIX_EPOCH + offset).await;
        let now = clock_clone.now();
        assert!(now >= UNIX_EPOCH + offset);
        tx.send(()).unwrap();
    });

    (task, rx)
}

#[tokio::test]
async fn test_multi_idle_to_mixed_time() {
    let clock = Arc::new(VirtualClock::from_epoch());
    clock.advance(Duration::from_secs(1));

    let (task1, done1) = spawn_idled_task(&clock, Duration::ZERO);
    let (task2, done2) = spawn_idled_task(&clock, Duration::from_secs(1));
    let (task3, mut done3) = spawn_idled_task(&clock, Duration::from_secs(2));

    tokio::time::timeout(Duration::from_secs(1), done1)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), done2)
        .await
        .unwrap()
        .unwrap();

    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut done3)
            .await
            .is_err(),
        "Task #3 should NOT have been finished (as its target is in the future)"
    );

    clock.advance(Duration::from_secs(1));
    tokio::time::timeout(Duration::from_secs(1), done3)
        .await
        .unwrap()
        .unwrap();

    try_join!(task1, task2, task3).unwrap();
}

#[tokio::test]
async fn test_backwards_advancement() {
    let clock = Arc::new(VirtualClock::from_epoch());
    clock.advance(Duration::from_secs(1));
    clock.advance_to(UNIX_EPOCH);

    assert_eq!(
        clock.now(),
        UNIX_EPOCH + Duration::from_secs(1),
        "VirtualClock should NEVER move backward"
    );
}

#[tokio::test]
async fn test_zero_duration_advancement() {
    let clock = Arc::new(VirtualClock::from_epoch());
    let now = clock.now();
    clock.advance(Duration::from_secs(0));

    assert_eq!(
        clock.now(),
        now,
        "Zero-based duration advancements should result in the same time"
    );
}

#[tokio::test]
async fn test_multi_advancement() {
    let clock = Arc::new(VirtualClock::from_epoch());
    let target = clock.now() + Duration::from_secs(2);

    let clock_clone = clock.clone();
    let task1 = tokio::spawn(async move {
        clock_clone.advance(Duration::from_secs(1));
    });

    let clock_clone = clock.clone();
    let task2 = tokio::spawn(async move {
        clock_clone.advance(Duration::from_secs(1));
    });

    try_join!(task1, task2).unwrap();
    assert_eq!(
        clock.now(),
        target,
        "Advancements should have summed up together"
    );
}
