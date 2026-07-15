use std::marker::PhantomData;
use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::define_event;
use std::time::Duration;

define_event!(OnTimeout, Duration);

pub trait DefaultTimeoutError: TaskError {
    fn default_timeout_error() -> Self;
}

impl DefaultTimeoutError for String {
    fn default_timeout_error() -> Self {
        "Timeout Occurred".to_string()
    }
}

#[doc(hidden)]
pub struct TimeoutMissingBuilder(());

#[doc(hidden)]
pub struct TimeoutPresentBuilder<T>(T);

pub struct TimeoutTaskFrame<T: TaskFrame> {
    frame: T,
    max_duration: Box<dyn Fn() -> Duration + Send + Sync>,
    error: Box<dyn Fn() -> T::Error + Send + Sync + 'static>,
}

pub struct TimeoutTaskFrameBuilder<T, TS, DS, ES> {
    frame: TS,
    max_duration: DS,
    error: ES,
    _marker: PhantomData<T>
}

impl<T: TaskFrame> TimeoutTaskFrame<T> {
    pub fn builder() -> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutMissingBuilder, TimeoutMissingBuilder> {
        TimeoutTaskFrameBuilder {
            frame: TimeoutMissingBuilder(()),
            max_duration: TimeoutMissingBuilder(()),
            error: TimeoutMissingBuilder(()),
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame, D, E> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, D, E> {
    pub fn frame(self, frame: T) -> TimeoutTaskFrameBuilder<T, TimeoutPresentBuilder<T>, D, E> {
        TimeoutTaskFrameBuilder {
            frame: TimeoutPresentBuilder(frame),
            max_duration: self.max_duration,
            error: self.error,
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame, TS, ES> TimeoutTaskFrameBuilder<T, TS, TimeoutMissingBuilder, ES> {
    pub fn duration(
        self,
        duration: Duration,
    ) -> TimeoutTaskFrameBuilder<T, TS, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync>>, ES> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: TimeoutPresentBuilder(Box::new(move || duration)),
            error: self.error,
            _marker: PhantomData,
        }
    }

    pub fn duration_fn<F>(
        self,
        f: impl Fn() -> Duration + Send + Sync + 'static,
    ) -> TimeoutTaskFrameBuilder<T, TS, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync>>, ES> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: TimeoutPresentBuilder(Box::new(f) as Box<dyn Fn() -> Duration + Send + Sync>),
            error: self.error,
            _marker: PhantomData
        }
    }
}

impl<T: TaskFrame, TS, DS> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutMissingBuilder> {
    pub fn error(
        self,
        error: T::Error,
    ) -> TimeoutTaskFrameBuilder<
        T,
        TS,
        DS,
        TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync>>,
    >
    where
        T::Error: Clone + Send + Sync + 'static,
    {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: self.max_duration,
            error: TimeoutPresentBuilder(Box::new(move || error.clone())),
            _marker: PhantomData,
        }
    }

    pub fn error_fn<F>(
        self,
        f: impl Fn() -> T::Error + Send + Sync + 'static,
    ) -> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync>>> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: self.max_duration,
            error: TimeoutPresentBuilder(Box::new(f) as Box<dyn Fn() -> T::Error + Send + Sync>),
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame> TimeoutTaskFrameBuilder<
    T,
    TimeoutPresentBuilder<T>,
    TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>,
    TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync + 'static>>
>
{
    pub fn build(self) -> TimeoutTaskFrame<T> {
        TimeoutTaskFrame {
            frame: self.frame.0,
            max_duration: self.max_duration.0,
            error: self.error.0,
        }
    }
}

impl<T: TaskFrame<Error: DefaultTimeoutError>> TimeoutTaskFrameBuilder<
    T,
    TimeoutPresentBuilder<T>,
    TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>,
    TimeoutMissingBuilder
>
{
    pub fn build(self) -> TimeoutTaskFrame<T> {
        TimeoutTaskFrame {
            frame: self.frame.0,
            max_duration: self.max_duration.0,
            error: Box::new(T::Error::default_timeout_error),
        }
    }
}

struct MissingTaskFrameParamError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutMissingBuilder, ES> {
    #[deprecated(note = "Missing required parameter for TaskFrame")]
    pub fn build(self, _err: MissingTaskFrameParamError) -> ! {
        panic!()
    }
}

struct MissingDurationParamError;
struct SpecifiedTaskFrameError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutPresentBuilder<T>, TimeoutMissingBuilder, ES> {
    #[deprecated(note = "Missing required parameter for Duration")]
    pub fn build(self, _err: MissingDurationParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for TaskFrame")]
    pub fn frame(self, _err: SpecifiedTaskFrameError) -> ! {
        panic!()
    }
}

struct SpecifiedDurationParamError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>, ES> {
    #[deprecated(note = "Missing required parameter for TaskFrame")]
    pub fn build(self, _err: MissingTaskFrameParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for Duration")]
    pub fn duration(self, _err: SpecifiedDurationParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for TaskFrame")]
    pub fn duration_fn(self, _err: SpecifiedDurationParamError) -> ! {
        panic!()
    }
}

struct SpecifiedErParamError;
impl<T: TaskFrame, TS, DS> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync + 'static>>> {
    #[deprecated(note = "Already specified parameter for error")]
    pub fn error(self, _err: SpecifiedErParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for error")]
    pub fn error_fn(self, _err: SpecifiedErParamError) -> ! {
        panic!()
    }
}

impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let duration = (self.max_duration)();
        let result = tokio::time::timeout(duration, self.frame.execute(ctx, &args)).await;

        if let Ok(inner) = result {
            return inner;
        }

        ctx.emit::<OnTimeout>(&duration).await;
        Err((self.error)())
    }
}