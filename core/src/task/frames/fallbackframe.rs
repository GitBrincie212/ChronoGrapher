use crate::define_event;
use crate::errors::{FallbackTaskFrameError, TaskError};
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
impl<T: TaskFrame, T2: TaskFrame> TaskFrame for FallbackTaskFrame<T, T2> {
    type Error = FallbackTaskFrameError<T2::Error>;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        match ctx.subdivide(&self.0).await {
            Err(err) => {
                ctx.emit::<OnFallbackEvent>(&(&err as &dyn TaskError)).await;
                ctx.subdivide(&self.1)
                    .await
                    .map_err(FallbackTaskFrameError::new)
            }

            Ok(()) => Ok(()),
        }
    }
}
