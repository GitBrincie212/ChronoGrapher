use crate::errors::TimeoutTaskFrameError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::define_event;
use std::time::Duration;

define_event!(OnTimeout, Duration);

enum TimeoutSource {
    Duration(Duration),
    Function(Box<dyn Fn() -> Duration + Send + Sync>),
}

pub struct TimeoutTaskFrame<T: TaskFrame> {
    frame: T,
    max_duration: TimeoutSource,
}

impl<T: TaskFrame> TimeoutTaskFrame<T> {
    pub fn new(frame: T, max_duration: Duration) -> Self {
        Self {
            frame,
            max_duration: TimeoutSource::Duration(max_duration),
        }
    }

    pub fn new_with(frame: T, function: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            frame,
            max_duration: TimeoutSource::Function(Box::new(function)),
        }
    }
}

impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = TimeoutTaskFrameError<T::Error>;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let duration = match &self.max_duration {
            TimeoutSource::Duration(dur) => *dur,
            TimeoutSource::Function(func) => func().clone(),
        };

        let result = tokio::time::timeout(duration, self.frame.execute(ctx, &args)).await;

        if let Ok(inner) = result {
            return inner.map_err(TimeoutTaskFrameError::Inner);
        }

        ctx.emit::<OnTimeout>(&duration).await;
        Err(TimeoutTaskFrameError::Timeout(duration))
    }
}
