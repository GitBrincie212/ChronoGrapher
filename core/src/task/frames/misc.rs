use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::TaskError;
use crate::task::TaskHookEvent;
#[allow(unused_imports)]
use crate::task::{ParallelTaskFrame, SequentialTaskFrame, TaskFrame};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

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
/// By default [`GroupedTaskFrameExecBehavior`] has 3 implementations of this trait. Those are:
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
#[async_trait]
pub trait GroupedTaskFramesExecBehavior: Send + Sync {
    async fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>>;
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
/// but also:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct GroupedTaskFramesQuitOnSuccess;

#[async_trait]
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesQuitOnSuccess {
    async fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        match result {
            Ok(()) => Some(Ok(())),
            Err(_) => None,
        }
    }
}

impl PersistenceObject for GroupedTaskFramesQuitOnSuccess {
    const PERSISTENCE_ID: &'static str =
        "chronographer::GroupedTaskFramesQuitOnSuccess#8895d78d-5552-464f-9e21-66771be396a5";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
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
/// but also:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`ParallelTaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct GroupedTaskFramesQuitOnFailure;

#[async_trait]
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesQuitOnFailure {
    async fn should_quit(&self, result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        match result {
            Ok(()) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl PersistenceObject for GroupedTaskFramesQuitOnFailure {
    const PERSISTENCE_ID: &'static str =
        "chronographer::GroupedTaskFramesQuitOnFailure#c2834489-7a63-4fda-bbc6-4ffe50b9733a";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
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
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct GroupedTaskFramesSilent;

#[async_trait]
impl GroupedTaskFramesExecBehavior for GroupedTaskFramesSilent {
    async fn should_quit(&self, _result: Result<(), TaskError>) -> Option<Result<(), TaskError>> {
        None
    }
}

impl PersistenceObject for GroupedTaskFramesSilent {
    const PERSISTENCE_ID: &'static str =
        "chronographer::GroupedTaskFramesSilent#59034b8a-d96e-4c42-933b-10d7aec14c88";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}

define_event!(
    /// [`OnChildTaskFrameStart`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnChildTaskFrameStart`]
    /// is ``TaskError`` which is the same error the inner primary TaskFrame returned
    ///
    /// # Constructor(s)
    /// When constructing a [`OnChildTaskFrameStart`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnChildTaskFrameStart::default`] or via simply pasting
    /// the struct name ([`OnChildTaskFrameStart`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnChildTaskFrameStart`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnChildTaskFrameStart`] is triggered when the [`ParallelTaskFrame`]'s / [`SequentialTaskFrame`]
    /// wrapped child [`TaskFrame`] started its execution
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnChildTaskFrameStart`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`ParallelTaskFrame`]
    /// - [`SequentialTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnChildTaskFrameStart, ()
);

define_event!(
    /// # Event Triggering
    /// [`OnChildTaskFrameStart`] is triggered when the [`ParallelTaskFrame`]'s / [`SequentialTaskFrame`]
    /// wrapped child [`TaskFrame`] ended its execution with a potential error
    ///
    /// # See Also
    /// - [`ParallelTaskFrame`]
    /// - [`SequentialTaskFrame`]
    OnChildTaskFrameEnd, Option<TaskError>
);

define_event_group!(
    /// [`ChildTaskFrameEvents`] is a marker trait, more specifically a [`TaskHookEvent`] group of
    /// [`TaskHookEvent`] (a system used closely with [`TaskHook`]). It contains no common payload type
    ///
    /// # Supertrait(s)
    /// Since it is a [`TaskHookEvent`] group, it requires every descended to implement the [`TaskHookEvent`],
    /// since no common payload type is present, any payload type is accepted
    ///
    /// # Trait Implementation(s)
    /// Currently, two [`TaskHookEvent`] implement the [`ChildTaskFrameEvents`] marker trait
    /// (event group). Those being [`OnChildTaskFrameStart`] and [`OnChildTaskFrameEnd`]
    ///
    /// # Object Safety
    /// [`ChildTaskFrameEvents`] is **NOT** object safe, due to the fact it implements the
    /// [`TaskHookEvent`] which itself is not object safe
    ///
    /// # See Also
    /// - [`OnChildTaskFrameStart`]
    /// - [`OnChildTaskFrameEnd`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    ChildTaskFrameEvents,
    OnChildTaskFrameStart, OnChildTaskFrameEnd
);
