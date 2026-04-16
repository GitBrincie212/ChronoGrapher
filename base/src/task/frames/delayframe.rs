use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group};
use std::time::Duration;
use tokio::time::Instant;

define_event!(OnDelayStart, Duration);

define_event!(OnDelayEnd, Duration);

define_event_group!(DelayEvents, Duration | OnDelayStart, OnDelayEnd);

pub struct DelayTaskFrame<T: TaskFrame> {
    frame: T,
    delay: Duration,
}

impl<T: TaskFrame> DelayTaskFrame<T> {
    pub fn new(frame: T, delay: Duration) -> Self {
        DelayTaskFrame { frame, delay }
    }
}

impl<T: TaskFrame> TaskFrame for DelayTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        ctx.emit::<OnDelayStart>(&self.delay).await;

        let deadline = Instant::now() + self.delay;
        tokio::time::sleep_until(deadline).await;

        ctx.emit::<OnDelayEnd>(&self.delay).await;

        self.frame.execute(ctx, args).await
    }
}
