use std::any::Any;
use std::fmt::{Debug, Display};
use thiserror::Error;

pub trait TaskError: Debug + Display + Send + Sync + 'static {
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

impl<T: Debug + Display + Send + Sync + Any> TaskError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error(
    "Task frame index `{index}` is out of bounds for `{src}` with task frame size `{size}` element(s)"
)]
pub struct TaskSelectionIndexOutOfBounds {
    pub index: usize,
    pub src: String,
    pub size: usize,
}

#[cfg(feature = "chrono")]
#[derive(Error, Debug, PartialEq, Eq)]
#[error("TimeDelta supplied is out of range (expected a positive TimeDelta value )")]
pub struct IntervalTimeDeltaOutOfRange;

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Floating-based seconds supplied is out of range")]
pub struct IntervalSecondsOutOfRange;
