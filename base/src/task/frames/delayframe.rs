use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group};
use std::time::Duration;

define_event!(OnDelayStart, Duration);

define_event!(OnDelayEnd, Duration);

define_event_group!(DelayEvents, Duration | OnDelayStart, OnDelayEnd);

enum DelaySource {
    Duration(Duration),
    Function(Box<dyn Fn() -> Duration + Send + Sync>),
}

pub struct DelayTaskFrame<T: TaskFrame> {
    frame: T,
    delay: DelaySource,
}

impl<T: TaskFrame> DelayTaskFrame<T> {
    pub fn new(frame: T, max_duration: Duration) -> Self {
        Self {
            frame,
            delay: DelaySource::Duration(max_duration),
        }
    }

    pub fn new_with(frame: T, function: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            frame,
            delay: DelaySource::Function(Box::new(function)),
        }
    }
}

impl<T: TaskFrame> TaskFrame for DelayTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let delay = match &self.delay {
            DelaySource::Duration(dur) => *dur,
            DelaySource::Function(func) => func().clone()
        };
        
        ctx.emit::<OnDelayStart>(&delay).await;
        tokio::time::sleep(delay).await;
        ctx.emit::<OnDelayEnd>(&delay).await;

        self.frame.execute(ctx, args).await
    }
}
