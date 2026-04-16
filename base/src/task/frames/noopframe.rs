use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct NoOperationTaskFrame<E, Args>(PhantomData<(E, Args)>);

impl<E: TaskError, Args: 'static + Send + Sync> Default for NoOperationTaskFrame<E, Args> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: TaskError, Args: 'static + Send + Sync> Clone for NoOperationTaskFrame<E, Args> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: TaskError, Args: 'static + Send + Sync> Copy for NoOperationTaskFrame<E, Args> {}

impl<E: TaskError, Args: 'static + Send + Sync> TaskFrame for NoOperationTaskFrame<E, Args> {
    type Error = E;
    type Args = Args;

    async fn execute(&self, _ctx: &TaskFrameContext, _args: &Self::Args) -> Result<(), Self::Error> {
        Ok(())
    }
}
