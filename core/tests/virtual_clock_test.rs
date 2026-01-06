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
    use chronographer::scheduler::DefaultSchedulerConfig;
    use chronographer::scheduler::clock::{
        AdvanceableSchedulerClock, SchedulerClock, VirtualClock,
    };
    use std::time::{Duration, UNIX_EPOCH};

    #[tokio::test]
    async fn test_initial_epoch() {
        let clock = VirtualClock::from_epoch();
        assert_approx!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            UNIX_EPOCH,
            EPSILON
        );
    }

    #[tokio::test]
    async fn test_custom_time() {
        let time0 = UNIX_EPOCH + Duration::from_secs(45);
        let clock = VirtualClock::new(time0);
        assert_approx!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            time0,
            EPSILON
        );
    }

    #[tokio::test]
    async fn test_advance() {
        let clock = VirtualClock::from_epoch();
        <VirtualClock as AdvanceableSchedulerClock<DefaultSchedulerConfig>>::advance(
            &clock,
            Duration::from_secs(1),
        )
        .await;
        assert_eq!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            UNIX_EPOCH + Duration::from_secs(1)
        );
        <VirtualClock as AdvanceableSchedulerClock<DefaultSchedulerConfig>>::advance(
            &clock,
            Duration::from_secs(100),
        )
        .await;
        assert_eq!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            UNIX_EPOCH + Duration::from_secs(101)
        );
    }

    #[tokio::test]
    async fn test_advance_to() {
        let clock = VirtualClock::from_epoch();
        let target = UNIX_EPOCH + Duration::from_secs(19);
        <VirtualClock as AdvanceableSchedulerClock<DefaultSchedulerConfig>>::advance_to(
            &clock, target,
        )
        .await;
        assert_approx!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            target,
            EPSILON
        );
        let target = UNIX_EPOCH + Duration::from_secs(235);
        <VirtualClock as AdvanceableSchedulerClock<DefaultSchedulerConfig>>::advance_to(
            &clock, target,
        )
        .await;
        assert_approx!(
            <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await,
            target,
            EPSILON
        );
    }

    #[tokio::test]
    async fn test_idle_to_simple_no_arc() {
        let clock = VirtualClock::from_epoch();
        let target = UNIX_EPOCH + Duration::from_secs(5);
        <VirtualClock as AdvanceableSchedulerClock<DefaultSchedulerConfig>>::advance(
            &clock,
            Duration::from_secs(5),
        )
        .await;
        <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::idle_to(&clock, target).await;
        let now = <VirtualClock as SchedulerClock<DefaultSchedulerConfig>>::now(&clock).await;
        assert_approx!(now, target, EPSILON);
    }
}
