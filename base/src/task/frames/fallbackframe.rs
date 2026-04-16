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

impl<T: TaskFrame, T2: TaskFrame> TaskFrame for FallbackTaskFrame<T, T2> {
    type Error = T2::Error;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        match self.0.execute(ctx).await {
            Err(err) => {
                ctx.emit::<OnFallbackEvent>(&(&err as &dyn TaskError)).await;
                self.1.execute(ctx).await
            }

            Ok(()) => Ok(()),
        }
    }
}
