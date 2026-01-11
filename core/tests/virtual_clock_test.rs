use std::time::Duration;

macro_rules! assert_approx {
    ($left: expr, $right: expr, $epsilon: expr) => {{
        let dur = match $right.duration_since($left) {
            Ok(dur) => dur,
            Err(e) => e.duration(),
        };

        assert!(dur <= $epsilon)
    }};
}

// A small value to avoid floating precision errors
pub const EPSILON: Duration = Duration::from_millis(1);

#[cfg(test)]
mod tests {
    use super::*;
    use chronographer::scheduler::clock::{
        AdvanceableSchedulerClock, SchedulerClock, VirtualClock,
    };
    use std::time::{Duration, UNIX_EPOCH};
    use chronographer::scheduler::DefaultSchedulerConfig;

    #[tokio::test]
    async fn test_initial_epoch() {
        let clock = VirtualClock::from_epoch();
        let scheduler_clock: &dyn SchedulerClock<DefaultSchedulerConfig> = &clock;
        assert_approx!(scheduler_clock.now().await, UNIX_EPOCH, EPSILON);
    }

    #[tokio::test]
    async fn test_custom_time() {
        let time0 = UNIX_EPOCH + Duration::from_secs(45);
        let clock = VirtualClock::new(time0);
        let scheduler_clock: &dyn SchedulerClock<DefaultSchedulerConfig> = &clock;
        assert_approx!(scheduler_clock.now().await, time0, EPSILON);
    }

    #[tokio::test]
    async fn test_advance() {
        let clock = VirtualClock::from_epoch();
        let scheduler_clock: &dyn SchedulerClock<DefaultSchedulerConfig> = &clock;
        let advanceable_clock: &dyn AdvanceableSchedulerClock<DefaultSchedulerConfig> = &clock;
        advanceable_clock.advance(Duration::from_secs(1)).await;
        assert_eq!(scheduler_clock.now().await, UNIX_EPOCH + Duration::from_secs(1));
        advanceable_clock.advance(Duration::from_secs(100)).await;
        assert_eq!(scheduler_clock.now().await, UNIX_EPOCH + Duration::from_secs(101));
    }

    #[tokio::test]
    async fn test_advance_to() {
        let clock = VirtualClock::from_epoch();
        let advanceable_clock: &dyn AdvanceableSchedulerClock<DefaultSchedulerConfig> = &clock;
        let scheduler_clock: &dyn SchedulerClock<DefaultSchedulerConfig> = &clock;
        let target = UNIX_EPOCH + Duration::from_secs(19);
        advanceable_clock.advance_to(target).await;
        assert_approx!(scheduler_clock.now().await, target, EPSILON);
        let target = UNIX_EPOCH + Duration::from_secs(235);
        advanceable_clock.advance_to(target).await;
        assert_approx!(scheduler_clock.now().await, target, EPSILON);
    }

    #[tokio::test]
    async fn test_idle_to_simple_no_arc() {
        let clock = VirtualClock::from_epoch();
        let target = UNIX_EPOCH + Duration::from_secs(5);
        let scheduler_clock: &dyn SchedulerClock<DefaultSchedulerConfig> = &clock;
        let advanceable_clock: &dyn AdvanceableSchedulerClock<DefaultSchedulerConfig> = &clock;
        advanceable_clock.advance(Duration::from_secs(5)).await;
        scheduler_clock.idle_to(target).await;
        let now = scheduler_clock.now().await;
        assert_approx!(now, target, EPSILON);
    }
}
