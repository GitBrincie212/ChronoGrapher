use crate::task::frames::CountingFrame;
use chronographer::task::FallbackTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

// This frame is used as fallback
// Decrement by one counter, opposite ass CountingFrame
struct FallbackCountingFrame {
    counter: Arc<AtomicUsize>,
    should_fail: bool,
}

impl TaskFrame for FallbackCountingFrame {
    type Error = String;
    type Args = String;

    async fn execute(
        &self,
        _ctx: &TaskFrameContext,
        _args: &Self::Args,
    ) -> Result<(), Self::Error> {
        // This print stmt can appear when usin the flag --no-capture
        println!("Fallback hit");
        self.counter.fetch_sub(1, Ordering::SeqCst);
        if self.should_fail {
            return Err("Fallback failed".to_string());
        }
        Ok(())
    }
}

macro_rules! init_counter_with_fallback {
    ($first_should_fail:expr, $second_should_fail:expr, $counter:expr) => {{
        let first = CountingFrame {
            counter: $counter.clone(),
            should_fail: $first_should_fail,
        };
        let second = FallbackCountingFrame {
            counter: $counter.clone(),
            should_fail: $second_should_fail,
        };

        let frame = FallbackTaskFrame::new(first, second);
        frame
    }};
}

#[tokio::test]
async fn fallback_execution_test() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = init_counter_with_fallback!(true, false, counter.clone());

    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Counter was incremented then decremented, so it should return to its original state (0)"
    );
}

#[tokio::test]
async fn no_fallback_primary_succeed() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = init_counter_with_fallback!(false, true, counter.clone());
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Counter should be incremented and not decremented by the fallback that should not run"
    );
}
#[tokio::test]
async fn both_primary_and_fallback_fail_returns_error() {
    let counter = Arc::new(AtomicUsize::new(0));

    let frame = init_counter_with_fallback!(true, true, counter.clone());
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(
        exec.is_err(),
        "Should propagate error when both primary and fallback fail"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Counter should be 0"
    );
}
