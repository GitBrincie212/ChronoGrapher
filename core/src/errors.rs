use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::time::Duration;
use thiserror::Error;

pub trait TaskError: Debug + Display + Send + Sync + 'static {
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

impl<T: Debug + Display + Send + Sync + Any> TaskError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[derive(Error, Debug)]
pub enum ConditionalTaskFrameError<T1: TaskError, T2: TaskError> {
    #[error(
        "ConditionalTaskFrame has failed, with the error originating from primary TaskFrame's failure:\n\t{0}"
    )]
    PrimaryFailed(T1),

    #[error(
        "ConditionalTaskFrame has failed, with the error originating from secondary TaskFrame's failure:\n\t{0}"
    )]
    SecondaryFailed(T2),

    #[error("ConditionalTaskFrame has returned false with `error_on_false` enabled")]
    TaskConditionFail,
}

#[derive(Error, Debug)]
pub enum TimeoutTaskFrameError<T: TaskError> {
    #[error(
        "TimeoutTaskFrame has failed, with the error originating from primary TaskFrame's failure:\n\t{0}"
    )]
    Inner(T),

    #[error("TimeoutTaskFrame has timeout with max duration '{0:?}'")]
    Timeout(Duration),
}

#[derive(Error, Debug)]
pub enum DependencyTaskFrameError<T: TaskError> {
    #[error(
        "DependencyTaskFrame has failed, with the error originating from inner TaskFrame's failure:\n\t{0}"
    )]
    Inner(T),

    #[error(
        "DependencyTaskFrame has failed with the error originating from the \"DependentFailBehavior\":\n\t'{0}'"
    )]
    DependenciesInvalidated(Box<dyn TaskError>),
}

macro_rules! newtype_error {
    ($name: ident) => {
        #[derive(Debug)]
        pub struct $name<T: TaskError>(T);
        impl<T: TaskError> $name<T> {
            pub fn new(error: T) -> Self {
                Self(error)
            }
        }

        impl<T: TaskError> Display for $name<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl<T: TaskError> Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

newtype_error!(DelayTaskFrameError);
newtype_error!(FallbackTaskFrameError);
newtype_error!(RetriableTaskFrameError);
newtype_error!(ThresholdTaskFrameError);

#[derive(Error, Debug)]
pub enum StandardCoreErrorsCG {
    #[error(
        "Task frame index `{0}` is out of bounds for `{1}` with task frame size `{2}` element(s)"
    )]
    TaskIndexOutOfBounds(usize, String, usize),

    #[error(
        "ConditionalTaskFrame returned false with error_on_false set to true, as such this error returns"
    )]
    TaskConditionFail,

    #[error("Dependencies have not been resolved")]
    TaskDependenciesUnresolved,

    #[error("{0}")]
    CronParserError(String),

    #[error("Timedelta supplied is out of range")]
    IntervalTimedeltaOutOfRange,

    #[error("Supplied TaskIdentifier `{0}` is non-existent in the current SchedulerTaskStore")]
    TaskIdentifierNonExistent(String),

    #[error("ThresholdTaskFrame's threshold has been surpassed")]
    ThresholdReachError,
}
