use std::error::Error;
use std::marker::PhantomData;
use crate::task::{TaskFrame, TaskFrameContext};
use async_trait::async_trait;

#[derive(Debug)]
pub struct NoOperationTaskFrame<E>(PhantomData<E>);

impl<E: Error + Send + Sync + 'static> Default for NoOperationTaskFrame<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: Error + Send + Sync + 'static> Clone for NoOperationTaskFrame<E> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<E: Error + Send + Sync + 'static> Copy for NoOperationTaskFrame<E> {}

#[async_trait]
impl<E: Error + Send + Sync + 'static> TaskFrame for NoOperationTaskFrame<E> {
    type Error = E;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        Ok(())
    }
}
