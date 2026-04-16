use crate::utils::macros::define_event;
use crate::errors::TimeoutTaskFrameError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use std::time::Duration;

define_event!(OnTimeout, Duration);

pub struct TimeoutTaskFrame<T: TaskFrame> {
    frame: T,
    max_duration: Duration,
}

impl<T: TaskFrame> TimeoutTaskFrame<T> {
    pub fn new(frame: T, max_duration: Duration) -> Self {
        Self {
            frame,
            max_duration,
        }
    }
}

impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = TimeoutTaskFrameError<T::Error>;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let result = tokio::time::timeout(
            self.max_duration, self.frame.execute(ctx, &args)
        ).await;

        if let Ok(inner) = result {
            return inner.map_err(TimeoutTaskFrameError::Inner);
        }

        ctx.emit::<OnTimeout>(&self.max_duration).await;
        Err(TimeoutTaskFrameError::Timeout(self.max_duration))
    }
}
