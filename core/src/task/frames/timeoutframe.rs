use crate::define_event;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::task::TaskFrame;
use async_trait::async_trait;
use std::time::Duration;
use crate::errors::TimeoutTaskFrameError;

define_event!(
    OnTimeout, Duration
);

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

#[async_trait]
impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = TimeoutTaskFrameError<T::Error>;
    
    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let result =
            tokio::time::timeout(self.max_duration, ctx.subdivide(&self.frame)).await;

        if let Ok(inner) = result {
            return inner.map_err(TimeoutTaskFrameError::Inner);
        }

        ctx.emit::<OnTimeout>(&self.max_duration).await;
        Err(TimeoutTaskFrameError::Timeout(self.max_duration))
    }
}
