use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use chronographer::task::{ErasedTaskFrame, TaskFrame, TaskFrameContext};

mod collectionframe_test;
mod condition_taskframe_test;
mod delay_taskframe_test;
mod dependency_taskframe_test;
mod dynamic_taskframe_test;
mod fallback_taskframe_test;
mod noop_operation_taskframe_test;
mod threshold_taskframe_test;
mod timeout_taskframe_test;

fn ok_frame(
    counter: &Arc<AtomicUsize>,
) -> Arc<dyn ErasedTaskFrame<()>> {
    Arc::new(CountingFrame {
        counter: counter.clone(),
        should_fail: false,
    })
}

fn failing_frame(
    counter: &Arc<AtomicUsize>,
) -> Arc<dyn ErasedTaskFrame<()>> {
    Arc::new(CountingFrame {
        counter: counter.clone(),
        should_fail: true,
    })
}

struct CountingFrame {
    counter: Arc<AtomicUsize>,
    should_fail: bool,
}

impl TaskFrame for CountingFrame {
    type Error = String;
    type Args = ();

    async fn execute(
        &self,
        _ctx: &TaskFrameContext,
        _args: &Self::Args,
    ) -> Result<(), Self::Error> {
        let counter = self.counter.clone();
        let should_fail = self.should_fail;

        counter.fetch_add(1, Ordering::SeqCst);
        if should_fail {
            return Err("TaskFrame Failed".to_owned());
        }

        Ok(())
    }
}