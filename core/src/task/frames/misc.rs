use crate::task::TaskError;

#[allow(unused_imports)]
use crate::task::{ParallelTaskFrame, SequentialTaskFrame, TaskFrame};

/// [`GroupedTaskFrameExecBehavior`] is a mechanism used in conjunction with [`ParallelTaskFrame`]
/// and [`SequentialTaskFrame`] **(we call them grouped task frames)**, it defines the behavior for
/// how it should behave according to the results of its child [`TaskFrame`]s
///
/// # Required Method(s)
/// When implementing the [`GroupedTaskFrameExecBehavior`], one has to supply an implementation
/// for [`GroupedTaskFrameExecBehavior::should_quit`], which evaluates if it should quit via an
/// ``Option<Result<(), TaskError>>``, where ``Some(...)`` indicates to quit with the result and ``None``
/// indicates not to quit with any result and continue the execution
///
/// # Trait Implementation(s)
/// By default [`GroupedTaskFrameExecBehavior`] has 3 implementations of this trait. Those being:
/// - [`GroupedTaskFramesQuitOnSuccess`] Quits the grouped task frame, if at least one child [`TaskFrame`] succeeds
/// - [`GroupedTaskFramesQuitOnFailure`] Quits the grouped task frame, if at least one child [`TaskFrame`] fails
/// - [`GroupedTaskFramesSilent`] Does not care about the results of every child [`TaskFrame`]
///
/// By default, [`ParallelTaskFrame`] and [`SequentialTaskFrame`] use [`GroupedTaskFramesQuitOnFailure`]
///
/// # Object Safety
/// [`GroupedTaskFramesExecBehavior`] trait is object safe as seen throughout [`SequentialTaskFrame`]'s
/// and [`ParallelTaskFrame`]'s source code
///
/// # See Also
/// - [`GroupedTaskFramesQuitOnSuccess`]
/// - [`GroupedTaskFramesQuitOnFailure`]
/// - [`GroupedTaskFramesSilent`]
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`TaskFrame`]
/// - [`GroupedTaskFrameExecBehavior::should_quit`]
pub trait GroupedTaskFramesExecBehavior: Send + Sync {
    fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>>;
}

/// [`GroupedTaskFramesQuitOnSuccess`] is an implementation of [`GroupedTaskFramesExecBehavior`] trait,
/// and when evaluated, it quits [`ParallelTaskFrame`] or [`SequentialTaskFrame`] if at least
/// one child has returned a success result
///
/// # Constructor(s)
/// [`GroupedTaskFramesQuitOnSuccess`] can be simply constructed via rust's default struct
/// initialization as there is no data attached to it, alternatively one can use
/// [`GroupedTaskFramesQuitOnSuccess::default`] via [`Default`]
///
/// # Trait Implementation(s)
/// Obviously, as discussed above, [`GroupedTaskFramesQuitOnSuccess`] implements [`GroupedTaskFramesExecBehavior`]
/// but it also implements [`Default`] (again as discussed), in addition, [`Clone`], [`Copy`] and [`Debug`]
///
/// # See Also
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
#[derive(Debug, Default, Clone, Copy)]
pub struct GroupedTaskFramesQuitOnSuccess;
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesQuitOnSuccess {
    fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        match result {
            Ok(()) => Some(Ok(())),
            Err(_) => None,
        }
    }
}

/// [`GroupedTaskFramesQuitOnFailure`] is an implementation of [`GroupedTaskFramesExecBehavior`] trait,
/// and when evaluated, it quits [`ParallelTaskFrame`] or [`SequentialTaskFrame`] if at least
/// one child has failed (returns a failure)
///
/// # Constructor(s)
/// [`GroupedTaskFramesQuitOnFailure`] can be simply constructed via rust's default struct
/// initialization as there is no data attached to it, alternatively one can use
/// [`GroupedTaskFramesQuitOnFailure::default`] via [`Default`]
///
/// # Trait Implementation(s)
/// Obviously, as discussed above, [`GroupedTaskFramesQuitOnFailure`] implements [`GroupedTaskFramesExecBehavior`]
/// but it also implements [`Default`] (again as discussed), in addition, [`Clone`], [`Copy`] and [`Debug`]
///
/// # See Also
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
pub struct GroupedTaskFramesQuitOnFailure;
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesQuitOnFailure {
    fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        match result {
            Ok(()) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

/// [`GroupedTaskFramesQuitOnFailure`] is an implementation of [`GroupedTaskFramesExecBehavior`] trait,
/// it continues execution of [`ParallelTaskFrame`] and [`SequentialTaskFrame`] no matter what result
/// a child [`TaskFrame`] returns, acts as an identity
///
/// # Constructor(s)
/// [`GroupedTaskFramesQuitOnFailure`] can be simply constructed via rust's default struct
/// initialization as there is no data attached to it, alternatively one can use
/// [`GroupedTaskFramesQuitOnFailure::default`] via [`Default`]
///
/// # Trait Implementation(s)
/// Obviously, as discussed above, [`GroupedTaskFramesQuitOnFailure`] implements [`GroupedTaskFramesExecBehavior`]
/// but it also implements [`Default`] (again as discussed), in addition, [`Clone`], [`Copy`] and [`Debug`]
///
/// # See Also
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
pub struct GroupedTaskFramesSilent;
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesSilent {
    fn should_quit(&self, _result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        None
    }
}
