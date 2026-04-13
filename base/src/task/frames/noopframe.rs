use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext};
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct NoOperationTaskFrame<E, A>(PhantomData<(E, A)>);

impl<E: TaskError, A> Default for NoOperationTaskFrame<E, A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: TaskError, A> Clone for NoOperationTaskFrame<E, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: TaskError, A> Copy for NoOperationTaskFrame<E, A> {}

#[async_trait]
impl<E: TaskError, A: Send + Sync + 'static> TaskFrame for NoOperationTaskFrame<E, A> {
    type Error = E;
    type Args = A;

    async fn execute(&self, _ctx: &TaskFrameContext, _args: &Self::Args) -> Result<(), Self::Error> {
        Ok(())
    }
}
