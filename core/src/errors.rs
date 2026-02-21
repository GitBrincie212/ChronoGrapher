use std::any::Any;
use std::fmt::{Debug, Display};
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

    #[error("The current SchedulerEngine does not support scheduler instructions")]
    SchedulerInstructionsUnsupported,
}
