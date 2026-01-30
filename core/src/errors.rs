use std::fmt::Debug;
use thiserror::Error;

#[allow(unused_imports)]
use crate::task::SelectTaskFrame;

#[allow(unused_imports)]
use crate::task::ConditionalFrame;

#[allow(unused_imports)]
use crate::task::DependencyTaskFrame;

#[allow(unused_imports)]
use crate::task::TimeoutTaskFrame;

#[allow(unused_imports)]
use crate::task::dependencyframe::DependentFailureOnFail;

/// [`ChronographerErrors`] is the main enum that contains all the errors which can be thrown by
/// ChronoGrapher, it uses under the hood [`thiserror`] to make it as smooth sailing to add more
/// errors in the future as possible. This enum is private only to the core package itself,
/// as no other is aware of the existence
#[derive(Error, Debug)]
pub enum ChronographerErrors {
    /// This error is meant to happen when retrieving an index from a container that has a
    /// specified length, but the index is out of bounds. In the core package it is mainly caused
    /// by [`SelectTaskFrame`]
    #[error(
        "Task frame index `{0}` is out of bounds for `{1}` with task frame size `{2}` element(s)"
    )]
    TaskIndexOutOfBounds(usize, String, usize),

    /// This error is meant to happen when [`ConditionalTaskFrame`]
    /// returns true and the flag ``error_on_false`` is set to true
    #[error(
        "ConditionalTaskFrame returned false with error_on_false set to true, as such this error returns"
    )]
    TaskConditionFail,

    /// This error is meant to happen when dependencies from [`DependencyTaskFrame`]
    /// aren't resolved with the dependent behavior set to [`DependentFailureOnFail`]
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
