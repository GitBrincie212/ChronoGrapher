use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext};
use async_trait::async_trait;

pub struct DynamicTaskFrame<T>(T);

impl<T, F, E> DynamicTaskFrame<T>
where
    T: (Fn(&TaskFrameContext) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: TaskError,
{
    pub fn new(func: T) -> Self {
        Self(func)
    }
}

#[async_trait]
impl<T, F, E> TaskFrame for DynamicTaskFrame<T>
where
    T: (Fn(&TaskFrameContext) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: TaskError,
{
    type Error = E;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        self.0(ctx).await
    }
}

#[macro_export]
macro_rules! dynamic_taskframe {
    ($block: block) => {{
        DynamicTaskFrame::new(|taskframe_ctx| async {
            $block;
            Ok(())
        })
    }};
}
