use crate::utils::macros::define_event;
use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use async_trait::async_trait;

define_event!(OnFallbackEvent, &'a dyn TaskError);

pub struct FallbackTaskFrame<T, T2>(T, T2);

impl<T: TaskFrame, T2: TaskFrame> FallbackTaskFrame<T, T2> {
    pub fn new(primary: T, secondary: T2) -> Self {
        Self(primary, secondary)
    }
}

#[async_trait]
impl<T: TaskFrame, T2: TaskFrame<Args = (T::Args, T::Error)>> TaskFrame for FallbackTaskFrame<T, T2> 
    where T::Args: Clone
{
    type Error = T2::Error;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        match ctx.subdivide(&self.0, args).await {
            Err(err) => {
                ctx.emit::<OnFallbackEvent>(&(&err as &dyn TaskError)).await;
                let secondary_args = (args.clone(), err);
                ctx.subdivide(&self.1, &secondary_args).await
            }

            Ok(()) => Ok(()),
        }
    }
}
