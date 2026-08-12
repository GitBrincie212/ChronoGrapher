use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, payload_wrapper};
use std::ops::Deref;

payload_wrapper!(
    /// A simple wrapper type of reference [`TaskError`] unable to be created from foreign code in order to prevent
    /// emissions of the [`OnFallback`] event from other sources and keeping things encapsulated.
    ///
    /// # See Also
    /// - [`OnFallback`] - The event which uses this wrapper as its payload.
    /// - [`FallbackTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnFallback`] event.
    FallbackError<'a>(&'a dyn TaskError)
);

define_event!(
    /// A [`TaskHookEvent`] triggered when the primary [`TaskFrame`] inside [`FallbackTaskFrame`] produces
    /// an error before running the secondary / fallback [`TaskFrame`].
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`FallbackTaskFrame`], it's the only place it is emitted
    /// after the primary [`TaskFrame`] errors out and before running the secondary / fallback [`TaskFrame`].
    ///
    /// # Payload Type
    /// The payload type consists of only one parameter that being [`FallbackError`] which is the configured
    /// the error produced from the primary [`TaskFrame`] (it can be turned into ``&dyn TaskError``)
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`FallbackTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own fallbacks (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`FallbackError`] - The primary [`TaskFrame's`] produced error.
    /// - [`FallbackTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`FallbackTaskFrame`]
    OnFallback, FallbackError<'a>
);

/// A type-alias of nested [`FallbackTaskFrames`] for easily providing two fallbacks.
/// For better ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow)
/// macro or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder).
/// Refer on both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
/// and [`FallbackTaskFrame`] respectively for more information.
///
/// # Semantic(s)
/// Allows for specifying this type alias for double fallbacks as opposed to nested fallbacks. In order
/// to construct a double fallback via base API, one can use [`FallbackTaskFrame::double`].
///
/// # Generic(s)
/// - ``T1`` The primary workflow to run first
/// - ``T2`` The secondary workflow (first fallback) to run after the primary workflow fails
/// - ``T3`` The tertiary workflow (second fallback) to run after the secondary workflow fails
///
/// # See Also
/// - [`FallbackTaskFrames`] - The [`TaskFrame`] responsible for this type alias.
/// - [`FallbackTaskFrame::double`] - The constructor for producing this.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`FallbackTaskFrames`]
pub type DoubleFallback<T1, T2, T3> = FallbackTaskFrame<T1, FallbackTaskFrame<T2, T3>>;

/// A type-alias of nested [`FallbackTaskFrames`] for easily providing three fallbacks.
/// For better ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow)
/// macro or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder).
/// Refer on both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
/// and [`FallbackTaskFrame`] respectively for more information.
///
/// # Semantic(s)
/// Allows for specifying this type alias for three fallbacks as opposed to nested fallbacks. In order
/// to construct a triplet fallback via base API, one can use [`FallbackTaskFrame::triplet`].
///
/// # Generic(s)
/// - ``T1`` The primary workflow to run first
/// - ``T2`` The secondary workflow (first fallback) to run after the primary workflow fails
/// - ``T3`` The tertiary workflow (second fallback) to run after the secondary workflow fails
/// - ``T4`` The quaternary workflow (third fallback) to run after the tertiary workflow fails
///
/// # See Also
/// - [`FallbackTaskFrames`] - The [`TaskFrame`] responsible for this type alias.
/// - [`FallbackTaskFrame::triplet`] - The constructor for producing this.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`FallbackTaskFrames`]
pub type TripleFallback<T1, T2, T3, T4> =
    FallbackTaskFrame<T1, FallbackTaskFrame<T2, FallbackTaskFrame<T3, T4>>>;

/// The [`FallbackTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// errors from its primary nested [`TaskFrame`] / workflow via a secondary [`TaskFrame`] / workflow.
///
/// # Decorating / Wrapping Behavior
/// When wrapping [`FallbackTaskFrame`] onto a primary workflow provided a secondary / fallback workflow.
/// It runs the primary workflow first. If it errors out, it triggers the secondary / fallback workflow
/// **which must accept the error value of the primary workflow as an argument (both workflows must also
/// have the identical error type)**.
///
/// The final result (success or failure) is determined by the secondary workflow. Whereas if the primary
/// workflow succeeds, the final result will always be success and never execute the secondary / fallback
/// workflow.
///
/// # Execution Error(s)
/// There are no pre-defined errors that [`FallbackTaskFrame`] throws, instead every error is thrown
/// by the secondary / fallback [`TaskFrame`] (usually it's the primary error)
///
/// # Events
/// The [`FallbackTaskFrame`] fires only one event that being [`OnFallback`] which is emitted when the
/// primary workflow fails with an error and before the secondary / fallback workflow runs. This event
/// contains as payload a reference to the primary error itself that being [`FallbackError`] which is a
/// thin-wrapper around a reference to [`TaskError`].
///
/// # Constructor(s)
/// When it comes to creating a [`FallbackTaskFrame`], one can use the various constructors depending
/// on the fallback count. [`FallbackTaskFrame::singular`] for one fallback, [`FallbackTaskFrame::double`]
/// for two fallback and [`FallbackTaskFrame::triplet`] for three fallbacks.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. as the
/// workflow primitive equivalent for [`FallbackTaskFrame`] inside the macro is ``fallback(...)`` which
/// accepts at least one fallback (apart from that can be any number of fallback).
///
/// When providing multiple fallbacks, each fallback is processed one by one from left to right with
/// the expansion using multiple [`FallbackTaskFrame`] nested types. For more information it's recommended
/// to check the [`workflow`](chronographer::prelude::workflow) macro itself.
///
/// Finally, you can use [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) and specify one or multiple times
/// the builder method [`TaskFrameBuilder::with_fallback`](chronographer::task::TaskFrameBuilder::with_fallback)
/// for each fallback, executing in order from bottom up.
///
/// # Trait Implementation(s)
/// Apart from [`TaskFrame`] which [`FallbackTaskFrame`] implements. There is no other prominent trait
/// which it currently implements.
///
/// # Example(s)
/// ```rust
/// use chronographer::prelude::*;
///
/// // Assume MyFallbackTaskFrame is defined with an arbitrary workflow
/// # #[taskframe]
/// # #[workflow(retry(2))]
/// # async fn MyFallbackTaskFrame(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
///
/// #[taskframe]
/// #[workflow(fallback(MyFallbackTaskFrame::workflow()))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
/// # let inner: FallbackTaskFrame<MyTaskFrame, RetriableTaskFrame<MyFallbackTaskFrame>> = MyTaskFrame::workflow();
/// ```
/// Wraps ``MyTaskFrame`` inside the ``fallback`` ([`FallbackTaskFrame`]) with a configured fallback
/// being the entire workflow of our ``MyFallbackTaskFrame``. The same script can be re-written in the Base
/// API as the following:
/// ```rust
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame and MyFallbackTaskFrame already like before.
/// # #[taskframe]
/// # #[workflow(retry(2))]
/// # async fn MyFallbackTaskFrame(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
/// #
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// let workflow = FallbackTaskFrame::singular(MyTaskFrame, MyFallbackTaskFrame::workflow())
/// # let inner: FallbackTaskFrame<MyTaskFrame, RetriableTaskFrame<MyFallbackTaskFrame>> = workflow;
/// ```
///
/// ---
///
/// When it comes to configuring multiple fallbacks to execute sequentially from failures, it's achieved
/// by simply appending more fallbacks (they can even be mixed with different constructors):
/// ```rust
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyFallbackTaskFrame1 and MyFallbackTaskFrame2
/// # #[taskframe]
/// # #[workflow(retry(2))]
/// # async fn MyFallbackTaskFrame1(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
/// #
/// # #[taskframe]
/// # #[workflow(timeout(5s))]
/// # async fn MyFallbackTaskFrame2(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
///
/// #[taskframe]
/// #[workflow(
///     fallback(MyFallbackTaskFrame1::single(), MyFallbackTaskFrame2::workflow())
/// )]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
///
/// # let inner: DoubleFallback<MyTaskFrame, RetriableTaskFrame<MyFallbackTaskFrame1>, MyFallbackTaskFrame2> = workflow;
/// ```
///
/// The same version in the base API requires slightly more boilerplate, involving the creations of nested
/// [`FallbackTaskFrame`] types (which the macro hid from us for ergonomics) as so:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame, MyFallbackTaskFrame1 and MyFallbackTaskFrame2 already from before.
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
/// # #[taskframe]
/// # #[workflow(retry(2))]
/// # async fn MyFallbackTaskFrame1(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
/// #
/// # #[taskframe]
/// # #[workflow(timeout(5s))]
/// # async fn MyFallbackTaskFrame2(ctx: &TaskFrameContext, error: String) -> Result<(), String> {
/// #     Ok(())
/// # }
///
/// let workflow = FallbackTaskFrame::double(
///     MyTaskFrame,
///     MyFallbackTaskFrame1::workflow(),
///     MyFallbackTaskFrame2::single()
/// );
///
/// # let inner: DoubleFallback<MyTaskFrame, RetriableTaskFrame<MyFallbackTaskFrame1>, MyFallbackTaskFrame2> = workflow;
/// ```
///
/// # See Also
/// - [`FallbackTaskFrame::singular`] - The constructor for configuring one fallback in Base API.
/// - [`FallbackTaskFrame::double`] - A more ergonomic constructor for two fallbacks in Base API.
/// - [`FallbackTaskFrame::triplet`] - An even more ergonomic constructor for three fallbacks in Base API.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow primitive simply
///   by the name of ``fallback(...)``.
/// - [`OnFallback`] - The event the [`FallbackTaskFrame`] fires when the primary workflow fails.
/// - [`TaskFrame`] - The core trait that [`FallbackTaskFrame`] implements and uses.
pub struct FallbackTaskFrame<T, T2>(T, T2);

impl<T: TaskFrame, T2: TaskFrame> FallbackTaskFrame<T, T2> {
    /// The constructor method used as one way to configure a [`FallbackTaskFrame`] instance with double
    /// fallbacks. For better ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow)
    /// macro or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on
    /// both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`FallbackTaskFrame`] respectively for more information.
    ///
    /// For more fallbacks, this method may be called additional times and wrap the resulting
    /// [`FallbackTaskFrame`] as the fallback of the next. Or by using the alternative constructors
    /// [`FallbackTaskFrame::double`] and [`FallbackTaskFrame::triplet`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, the former being ``primary`` which is the primary workflow
    /// to run first whereas the latter is ``secondary`` which is the secondary / fallback workflow
    /// to execute after the primary workflow has failed.
    ///
    /// Do note the second workflow must be able to accept as an argument the first workflow's error
    /// type in order to act-upon the error.
    ///
    /// # Returns
    /// The new instance with the configured primary workflow being ``primary`` and the secondary workflow
    /// being ``secondary``.
    ///
    /// # See Also
    /// - [`FallbackTaskFrame`] - The main type the constructor is building.
    /// - [`FallbackTaskFrame::double`] - An alternative constructor for double fallbacks.
    /// - [`FallbackTaskFrame::triplet`] - An alternative constructor for triple fallbacks.
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`FallbackTaskFrame`]
    pub fn singular(primary: T, secondary: T2) -> Self {
        Self(primary, secondary)
    }

    /// A slightly more ergonomic constructor of [`FallbackTaskFrame::singular`] for double fallbacks. For better
    /// ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow) macro or
    /// by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on both
    /// [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`FallbackTaskFrame`] respectively for more information.
    ///
    /// # Argument(s)
    /// This method accepts three arguments:
    /// - ``primary`` The primary workflow to run first
    /// - ``secondary`` The secondary workflow (first fallback) to run after the primary workflow has failed.
    /// - ``tertiary`` The tertiary workflow (second fallback) to run after everything else has failed.
    ///
    /// Do note each fallback must accept as argument the previous fallback's error. **Additionally
    /// the primary error is not retained for tertiary to take action against.** You can work around this
    /// with [`TaskHooks`] or by retaining the information of the primary error via its error type.
    ///
    /// # Returns
    /// The new instance with the configured primary workflow being ``primary`` and the secondary workflow
    /// being ``secondary`` plus an additional fallback being ``tertiary``.
    ///
    /// # See Also
    /// - [`FallbackTaskFrame`] - The main type the constructor is building
    /// - [`FallbackTaskFrame::singular`] - An alternative constructor for one fallback.
    /// - [`FallbackTaskFrame::triplet`] - An alternative constructor for triple fallbacks.
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`FallbackTaskFrame`]
    pub fn double<T3: TaskFrame<Args = T2::Error>>(
        primary: T,
        secondary: T2,
        tertiary: T3,
    ) -> DoubleFallback<T, T2, T3> {
        FallbackTaskFrame::singular(primary, FallbackTaskFrame::singular(secondary, tertiary))
    }

    /// A slightly more ergonomic constructor of [`FallbackTaskFrame::double`] for triple fallbacks. For better
    /// ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow) macro or
    /// by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on both
    /// [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`FallbackTaskFrame`] respectively for more information.
    ///
    /// # Argument(s)
    /// This method accepts three arguments:
    /// - ``primary`` The primary workflow to run first
    /// - ``secondary`` The secondary workflow (first fallback) to run after the primary workflow has failed.
    /// - ``tertiary`` The tertiary workflow (second fallback) to run after the secondary workflow has failed.
    /// - ``quaternary`` The quaternary workflow (third fallback) to run after everything else has failed.
    ///
    /// Do note each fallback must accept as argument the previous fallback's error. **Additionally
    /// the primary and secondary error is not retained for tertiary and quaternary to take action against
    /// respectively.** You can work around this with [`TaskHooks`] or by retaining the information of the
    /// primary & secondary error via its error type.
    ///
    /// # Returns
    /// The new instance with the configured primary workflow being ``primary`` and the secondary workflow
    /// being ``secondary`` plus two additional fallbacks being ``tertiary`` and ``quaternary``.
    ///
    /// # See Also
    /// - [`FallbackTaskFrame`] - The main type the constructor is building
    /// - [`FallbackTaskFrame::singular`] - An alternative constructor for one fallback.
    /// - [`FallbackTaskFrame::double`] - An alternative constructor for double fallbacks.
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`FallbackTaskFrame`]
    pub fn triplet<T3: TaskFrame<Args = T2::Error>, T4: TaskFrame<Args = T3::Error>>(
        primary: T,
        secondary: T2,
        tertiary: T3,
        quaternary: T4,
    ) -> TripleFallback<T, T2, T3, T4> {
        FallbackTaskFrame::singular(
            primary,
            FallbackTaskFrame::double(secondary, tertiary, quaternary),
        )
    }
}

impl<T, T2> TaskFrame for FallbackTaskFrame<T, T2>
where
    T: TaskFrame,
    T2: TaskFrame<Args = T::Error>,
{
    type Error = T2::Error;
    type Args = T::Args;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        match self.0.execute(ctx, args).await {
            Err(err) => {
                ctx.emit::<OnFallback>(&FallbackError(&err as &dyn TaskError))
                    .await;
                self.1.execute(ctx, &err).await
            }

            Ok(()) => Ok(()),
        }
    }
}
