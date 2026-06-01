use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use chronographer::prelude::TaskFrameContext;
use chronographer::task::TaskFrame;

#[derive(Default, Clone)]
pub struct CountingTaskFrame {
    success_counter: Arc<AtomicUsize>,
    failure_counter: Arc<AtomicUsize>,
    error: Arc<AtomicBool>
}

impl CountingTaskFrame {
    pub fn identity(&self) -> usize {
        self.success_counter.load(Ordering::Acquire)
            + self.failure_counter.load(Ordering::Acquire)
    }

    pub fn successes(&self) -> usize {
        self.success_counter.load(Ordering::Acquire)
    }

    pub fn failures(&self) -> usize {
        self.failure_counter.load(Ordering::Acquire)
    }

    pub fn enable_failure(&self) {
        self.error.store(true, Ordering::Release);
    }

    pub fn disable_failure(&self) {
        self.error.store(false, Ordering::Release);
    }
}

impl TaskFrame for CountingTaskFrame {
    type Error = String;
    type Args = ();
    type Workflow = Self;

    async fn execute(&self, _: &TaskFrameContext, _: &Self::Args) -> Result<(), Self::Error> {
        if self.error.load(Ordering::Acquire) {
            self.failure_counter.fetch_add(1, Ordering::SeqCst);
            return Err("Dummy-based error used for unit tests".to_owned())
        }

        self.success_counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}