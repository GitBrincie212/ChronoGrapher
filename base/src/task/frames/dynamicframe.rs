use std::marker::PhantomData;
use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext};

pub struct DynamicTaskFrame<T, Args>(T, PhantomData<Args>);

impl<T, F, E, Args> DynamicTaskFrame<T, Args>
where
    T: (Fn(&TaskFrameContext, &Args) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: TaskError,
    Args: Send + Sync + 'static
{
    pub fn new(func: T) -> Self {
        Self(func, PhantomData)
    }
}

impl<T, F, E, Args> TaskFrame for DynamicTaskFrame<T, Args>
where
    T: (Fn(&TaskFrameContext, &Args) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: TaskError,
    Args: Send + Sync + 'static
{
    type Error = E;
    type Args = Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        self.0(ctx, args).await
    }
}