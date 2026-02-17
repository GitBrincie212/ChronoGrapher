use std::marker::PhantomData;
use crate::task::{TaskFrame, TaskFrameContext};
use async_trait::async_trait;
use crate::errors::TaskError;

#[derive(Debug)]
pub struct NoOperationTaskFrame<E>(PhantomData<E>);

impl<E: TaskError> Default for NoOperationTaskFrame<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: TaskError> Clone for NoOperationTaskFrame<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: TaskError> Copy for NoOperationTaskFrame<E> {}

#[async_trait]
impl<E: TaskError> TaskFrame for NoOperationTaskFrame<E> {
    type Error = E;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        Ok(())
    }
}
