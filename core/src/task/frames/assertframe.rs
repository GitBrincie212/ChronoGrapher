use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;

pub struct AssertTaskFrame<T: TaskFrame> {
    frame: Arc<T>,
    on_error: Arc<dyn Fn() -> TaskError + Send + Sync>,
}

impl<T: TaskFrame> AssertTaskFrame<T> {
    pub fn new<F>(frame: T, on_error: F) -> Self
    where
        F: Fn() -> TaskError + Send + Sync + 'static,
    {
        Self {
            frame: Arc::new(frame),
            on_error: Arc::new(on_error),
        }
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for AssertTaskFrame<T> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        match self.frame.execute(ctx).await {
            Ok(()) => Err((self.on_error)()),
            Err(_) => Ok(()),
        }
    }
}
