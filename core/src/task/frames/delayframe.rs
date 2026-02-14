use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::task::TaskFrame;
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::Instant;
use crate::errors::DelayTaskFrameError;

define_event!(
    OnDelayStart, Duration
);

define_event!(
    OnDelayEnd, Duration
);

define_event_group!(
    DelayEvents, Duration |
    OnDelayStart, OnDelayEnd
);

pub struct DelayTaskFrame<T: TaskFrame> {
    frame: T,
    delay: Duration,
}

impl<T: TaskFrame> DelayTaskFrame<T> {
    pub fn new(frame: T, delay: Duration) -> Self {
        DelayTaskFrame {
            frame,
            delay,
        }
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for DelayTaskFrame<T> {
    type Error = DelayTaskFrameError<T::Error>;
    
    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        ctx.emit::<OnDelayStart>(&self.delay).await;
        
        let deadline = Instant::now() + self.delay;
        tokio::time::sleep_until(deadline).await;
        
        ctx.emit::<OnDelayEnd>(&self.delay).await;
        
        ctx.subdivide(&self.frame).await
            .map_err(DelayTaskFrameError::new)
    }
}
