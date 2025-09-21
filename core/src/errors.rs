use crate::task::TaskError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChronographerErrors {
    #[error("`{0}` Failed to successfully execute, the function returned an error: {1:?}")]
    FailedExecution(String, TaskError),

    #[error("`{0}` was aborted")]
    TaskAborted(String),

    #[error(
        "Task frame index `{0}` is out of bounds for SelectFrame with task frame size `{1}` element(s)"
    )]
    TaskIndexOutOfBounds(usize, usize),

    #[error(
        "ConditionalTaskFrame returned false with on_error set to true, as such this error returns"
    )]
    TaskConditionFail,

    #[error("Dependencies have not been resolved")]
    TaskDependenciesUnresolved,

    #[error("`{0}` Timed out")]
    TimeoutError(String),
}
