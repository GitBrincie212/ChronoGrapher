use chronolog_core::clock::AdvanceableScheduleClock;
use chronolog_core::clock::SchedulerClock;
use chronolog_core::clock::VirtualClock;
macro_rules! assert_approx {
    ($left: expr, $right: expr, $epsilon: expr) => {{
        let dur = match $right.duration_since($left) {
            Ok(dur) => dur,
            Err(e) => e.duration(),
        };

        assert!(dur <= $epsilon)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[tokio::test]
    async fn test_initial_epoch() {
        let clock = VirtualClock::from_epoch();
        assert_approx!(clock.now().await, UNIX_EPOCH, Duration::from_millis(1));
    }
    #[tokio::test]
    async fn test_custom_time() {
        let time0 = UNIX_EPOCH + Duration::from_secs(45);
        let clock = VirtualClock::new(time0);
        assert_approx!(clock.now().await, time0, Duration::from_millis(1));
    }
    #[tokio::test]
    async fn test_advance() {
        let clock = VirtualClock::from_epoch();
        clock.advance(Duration::from_secs(1)).await;
        assert_eq!(clock.now().await, UNIX_EPOCH + Duration::from_secs(1));
        clock.advance(Duration::from_secs(100)).await;
        assert_eq!(clock.now().await, UNIX_EPOCH + Duration::from_secs(101));
    }
    #[tokio::test]
    async fn test_advance_to() {
        let clock = VirtualClock::from_epoch();
        let target = UNIX_EPOCH + Duration::from_secs(19);
        clock.advance_to(target).await;
        assert_approx!(clock.now().await, target, Duration::from_millis(1));
    }

    #[tokio::test]
    async fn test_idle_to_simple_no_arc() {
        let clock = VirtualClock::from_epoch();
        let target = UNIX_EPOCH + Duration::from_secs(5);
        clock.advance(Duration::from_secs(5)).await;
        clock.idle_to(target).await;
        let now = clock.now().await;
        assert_approx!(now, target, Duration::from_millis(1));
    }
}
