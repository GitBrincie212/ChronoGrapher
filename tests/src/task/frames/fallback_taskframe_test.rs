use crate::impl_counting_frame;
use chronographer::task::FallbackTaskFrame;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fallback error")
    }
}

impl_counting_frame!(DummyError);

// This frame is used as fallback
// Decrement by one counter, opposite ass CountingFrame
struct FallbackCountingFrame {
    counter: Arc<AtomicUsize>,
}

impl TaskFrame for FallbackCountingFrame {
    type Error = DummyError;
    type Args = DummyError;

    async fn execute(
        &self,
        _ctx: &TaskFrameContext,
        _args: &Self::Args,
    ) -> Result<(), Self::Error> {
        // This print stmt can appear when usin the flag --no-capture
        println!("Fallback hit");
        self.counter.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn fallback_execution_test() {
    let counter = Arc::new(AtomicUsize::new(0));

    let first = CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    };

    let second = FallbackCountingFrame {
        counter: counter.clone(),
    };

    let frame = FallbackTaskFrame::new(first, second);
    let task = Task::new(TaskScheduleImmediate, frame);
    let exec = task.into_erased().run().await;

    assert!(exec.is_ok());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Counter was incremented then decremented, so it should return to its original state (0)"
    );
}
