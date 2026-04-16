use crate::utils::macros::define_event;
use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};

define_event!(OnFallbackEvent, &'a dyn TaskError);

pub struct FallbackTaskFrame<T, T2>(T, T2);

impl<T: TaskFrame, T2: TaskFrame> FallbackTaskFrame<T, T2> {
    pub fn new(primary: T, secondary: T2) -> Self {
        Self(primary, secondary)
    }
}

impl<T, T2> TaskFrame for FallbackTaskFrame<T, T2>
where
    T: TaskFrame,
    T2: TaskFrame<Args = T::Error>
{
    type Error = T2::Error;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        match self.0.execute(ctx, args).await {
            Err(err) => {
                ctx.emit::<OnFallbackEvent>(&(&err as &dyn TaskError)).await;
                self.1.execute(ctx, &err).await
            }

            Ok(()) => Ok(()),
        }
    }
}
