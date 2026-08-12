use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::noopframe::NoOperationTaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group};
use typed_builder::TypedBuilder;

/// The [`ConditionalFramePredicate`] trait allows the specification of a predicate function used in
/// [`ConditionalTaskFrame`], returning a boolean to decide to run the primary workflow.
///
/// # Required Method(s)
/// When it comes to implementing this trait the only required method is
/// [`ConditionalFramePredicate::check`] which is the main logic of the predicate.
///
/// # Implementation(s)
/// The only implementation of [`ConditionalFramePredicate`] is function pointers which
/// return a boolean value.
///
/// # Object Safety / Dynamic Dispatching
/// This trait is object safe (dyn compatible).
///
/// # See Also
/// - [`ConditionalTaskFrame`] - The [`TaskFrame`] responsible for using this trait
pub trait ConditionalFramePredicate: Send + Sync + 'static {
    /// The main logic for the predicate. View [`ConditionalFramePredicate`] for more info.
    fn check(&self) -> bool;
}

impl ConditionalFramePredicate for fn() -> bool {
    fn check(&self) -> bool {
        self()
    }
}

/// A simple wrapper type of ``()`` for no arguments unable to be created from foreign code in order to prevent
/// emissions of the [`OnTruthyValueEvent`] and [`OnFalseyValueEvent`] events from other sources and keeping
/// things encapsulated.
///
/// # See Also
/// - [`OnTruthyValueEvent`] - One of the events which uses this wrapper as its payload.
/// - [`OnFalseyValueEvent`] - The other event which uses this wrapper as its payload.
/// - [`ConditionalFramePredicate`] - The [`TaskFrame`] responsible for emitting the event.
pub struct NoPredicateArguments(());

define_event!(
    /// A [`TaskHookEvent`] triggered when the predicate inside [`ConditionalTaskFrame`] returns true.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`ConditionalTaskFrame`], it's the only place it is emitted
    /// after the predicate returns a truthy value
    ///
    /// # Payload Type
    /// There isn't any payload data associated with this event.
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`ConditionalTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event if you plan to emit it.
    ///
    /// # See Also
    /// - [`ConditionalFramePredicate`] - The predicate which is directly responsible.
    /// - [`ConditionalTaskFrame`] - The [`TaskFrame`] (indirectly) responsible for emitting the event.
    /// - [`OnFalseyValueEvent`] - The counterpart of this event which emits when the predicate returns false.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`ConditionalTaskFrame`].
    OnTruthyValueEvent, NoPredicateArguments
);

define_event!(
    /// A [`TaskHookEvent`] triggered when the predicate inside [`ConditionalTaskFrame`] returns false.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`ConditionalTaskFrame`], it's the only place it is emitted
    /// after the predicate returns a falsey value
    ///
    /// # Payload Type
    /// There isn't any payload data associated with this event.
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`ConditionalTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event if you plan to emit it.
    ///
    /// # See Also
    /// - [`ConditionalFramePredicate`] - The predicate which is directly responsible.
    /// - [`ConditionalTaskFrame`] - The [`TaskFrame`] (indirectly) responsible for emitting the event.
    /// - [`OnFalseyValueEvent`] - The counterpart of this event which emits when the predicate returns true.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`ConditionalTaskFrame`].
    OnFalseyValueEvent, NoPredicateArguments
);

define_event_group!(
    /// A closed-form [`TaskHookEvent`] group (THEG) consisting of [`OnTruthyValueEvent`] and [`OnFalseyValueEvent`]
    /// as the events it hosts.
    ///
    /// # Common Payload Type
    /// The common payload type requires for there not to be any payload data associated.
    ///
    /// # Is Emittable?
    /// Since the events are intended for only [`ConditionalTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event and THEG for your own conditionals (or anything else) if you plan to emit it.
    ///
    /// # Supported Events
    /// The events which this THEG supports are [`OnTruthyValueEvent`] and [`OnFalseyValueEvent`] for
    /// listening to the predicate returning a truthy and falsey value respectively.
    ///
    /// # See Also
    /// - [`OnTruthyValueEvent`] - A child event of the THEG which is emitted when the predicate returns true.
    /// - [`OnFalseyValueEvent`] - A child event of the THEG which is emitted when the predicate returns false.
    /// - [`ConditionalTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (the subtrait) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`ConditionalTaskFrame`]
    ConditionalPredicateEvents,
    NoPredicateArguments | OnTruthyValueEvent,
    OnFalseyValueEvent
);

#[allow(private_interfaces)]
trait NoOperationTaskFrameBound: Default {}
impl<E: TaskError, Args: 'static + Send + Sync> NoOperationTaskFrameBound
    for NoOperationTaskFrame<E, Args>
{
}

/// The [`ConditionalTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// conditional logic for the [`TaskFrame`] / workflow via a predicate.
///
/// # Decorating / Wrapping Behavior
/// When wrapping [`ConditionalTaskFrame`] onto the workflow provided a predicate ([`ConditionalFramePredicate`]).
/// It first checks what boolean value the predicate returns.
///
/// On a truthy value, it executes the workflow as usual, whereas on a falsey value depending on its
/// configuration, it can execute a backup workflow.
///
/// # Execution Error(s)
/// There are no pre-defined errors that [`ConditionalTaskFrame`] throws, instead every error is thrown
/// by the [`TaskFrames`] / workflows themselves (both primary & backup).
///
/// # Events
/// The [`ConditionalTaskFrame`] fires only two events those being [`OnTruthyValueEvent`] and [`OnFalseyValueEvent`]
/// both fired after the predicate returns the boolean value, the former fires when the value is truthy
/// whereas the latter fires only when the value is falsey
///
/// # Constructor(s)
/// When it comes to creating a [`ConditionalTaskFrame`], one of the ways to create it is via
/// [`ConditionalTaskFrame::builder`] with an optional backup.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. as the
/// workflow primitive equivalent for [`ConditionalTaskFrame`] inside the macro is ``condition(...)`` which
/// accepts the predicate and optionally a fallback.
///
/// # Trait Implementation(s)
/// Apart from [`TaskFrame`] which [`ConditionalTaskFrame`] implements. There is no other prominent trait
/// which it currently implements.
///
/// # Example(s)
/// ```rust
/// use chronographer::prelude::*;
///
/// fn my_predicate() -> bool {
///     true
/// }
///
/// #[taskframe]
/// #[workflow(condition(my_predicate))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
/// # let inner: NonBackupConditionalTaskFrame<MyTaskFrame> = MyTaskFrame::workflow();
/// ```
/// Wraps ``MyTaskFrame`` inside the ``condition`` ([`ConditionalTaskFrame`]) with a configured predicate.
/// The same script can be re-written in the Base API as the following:
/// ```rust
/// use chronographer::prelude::*;
///
/// /// // Assume we have defined our predicate and MyTaskFrame already like before.
/// # fn my_predicate() -> bool {
/// #     true
/// # }
/// #
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// let workflow = ConditionalTaskFrame::builder()
///     .frame(MyTaskFrame)
///     .predicate(my_predicate as fn() -> bool)
///     .build();
///
/// # let inner: NonBackupConditionalTaskFrame<MyTaskFrame> = workflow;
/// ```
///
/// ---
///
/// When it comes to configuring a backup [`TaskFrame`] / workflow to run on falsey values from the predicate.
/// You can simply specify them as follows:
/// ```rust
/// use chronographer::prelude::*;
///
/// /// // Assume we have defined our predicate.
/// # fn my_predicate() -> bool {
/// #     false
/// # }
///
/// #[taskframe]
/// async fn MyBackupTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
///
/// #[taskframe]
/// #[workflow(condition(my_predicate, backup = MyBackupTaskFrame))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
///
/// # let inner: ConditionalTaskFrame<MyTaskFrame, MyBackupTaskFrame> = MyTaskFrame::workflow();
/// ```
///
/// The same version in the base API is made like so:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined the predicate, MyTaskFrame & MyBackupTaskFrame already from before.
/// # fn my_predicate() -> bool {
/// #     false
/// # }
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
/// # #[taskframe]
/// # async fn MyBackupTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #     Ok(())
/// # }
///
/// let workflow = ConditionalTaskFrame::builder()
///     .frame(MyTaskFrame)
///     .backup(MyBackupTaskFrame)
///     .predicate(my_predicate as fn() -> bool)
///     .build();
///
/// # let inner: ConditionalTaskFrame<MyTaskFrame, MyBackupTaskFrame> = workflow;
/// ```
///
/// # See Also
/// - [`ConditionalTaskFrame::builder`] - A constructor for configuring a conditional in Base API.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow primitive simply
///   by the name of ``condition(...)``.
/// - [`OnTruthyValueEvent`] - The event which fires when the predicate returns a truthy value.
/// - [`OnFalseyValueEvent`] - The event which fires when the predicate returns a falsey value.
/// - [`TaskFrame`] - The core trait that [`ConditionalTaskFrame`] implements and uses.
#[derive(TypedBuilder)]
pub struct ConditionalTaskFrame<T1, T2 = NoOperationTaskFrame<<T1 as TaskFrame>::Error>>
where
    T1: TaskFrame<Args = ()>,
    T2: TaskFrame<Error = T1::Error, Args = ()>,
{
    /// The builder method which sets the primary [`TaskFrame`] / workflow.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`TaskFrame`] which is the primary workflow.
    ///
    /// # Returns
    /// This method returns the [`ConditionalTaskFrameBuilder`] configured with the specified
    /// primary [`TakFrame`] / workflow to chain more builder methods if needed and build the [`ConditionalTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call ``.build()``
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`ConditionalTaskFrame`] uses.
    /// - [`ConditionalTaskFrame`] - The final result of the builder.
    frame: T1,

    /// The builder method which sets the backup [`TaskFrame`] / workflow.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`TaskFrame`] which is the backup workflow.
    ///
    /// # Returns
    /// This method returns the [`ConditionalTaskFrameBuilder`] configured with the specified
    /// backup [`TakFrame`] / workflow to chain more builder methods if needed and build the [`ConditionalTaskFrame`].
    ///
    /// # Default Value
    /// Depending on whenever or not ``T2`` (backup workflow) implements the default trait, it will
    /// use the default value from it, for [`ConditionalTaskFrame`] with no backup, its default value
    /// utilizes [`NoOperationTaskFrame`] (does nothing).
    ///
    /// On the other hand, if ``T2`` is specified to be some other ``TaskFrame``, then it must
    /// be specified via the builder method.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`ConditionalTaskFrame`] uses.
    /// - [`ConditionalTaskFrame`] - The final result of the builder.
    #[builder(default, default_where(T2: NoOperationTaskFrameBound))]
    backup: T2,

    /// The builder method which sets the predicate itself dictating whenever or not
    /// [`ConditionalTaskFrame`] should run the primary [`TaskFrame`] / workflow.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`ConditionalFramePredicate`] which implements the custom
    /// predicate logic to dictate whenever to run the primary workflow or not.
    ///
    /// # Returns
    /// This method returns the [`ConditionalTaskFrameBuilder`] configured with the specified predicate
    /// to chain more builder methods if needed and build the [`ConditionalTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call ``.build()``
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`ConditionalFramePredicate`] - The predicate logic of [`ConditionalTaskFrame`]
    /// - [`ConditionalTaskFrame`] - The final result of the builder.
    #[builder(setter(transform = |s: impl ConditionalFramePredicate| {
        Box::new(s) as Box<dyn ConditionalFramePredicate>
    }))]
    predicate: Box<dyn ConditionalFramePredicate>,
}

/// A type-alias of a [`ConditionalTaskFrame`] with no backup workflow specified. It exists primarily
/// due to Rust type inference limitations (while initially seems obvious what ``T2`` parameter is, the
/// compiler cannot really see).
///
/// # See Also
/// - [`ConditionalTaskFrame`] - The [`TaskFrame`] responsible for this type alias.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`ConditionalTaskFrame`]
pub type NonBackupConditionalTaskFrame<T> =
    ConditionalTaskFrame<T, NoOperationTaskFrame<<T as TaskFrame>::Error>>;

impl<T1, T2> TaskFrame for ConditionalTaskFrame<T1, T2>
where
    T1: TaskFrame<Args = ()>,
    T2: TaskFrame<Args = (), Error = T1::Error>,
{
    type Error = T1::Error;
    type Args = ();

    async fn execute(&self, ctx: &TaskFrameContext, _args: &Self::Args) -> Result<(), Self::Error> {
        let result = self.predicate.check();

        if result {
            ctx.emit::<OnTruthyValueEvent>(&NoPredicateArguments(()))
                .await;
            return self.frame.execute(ctx, &()).await;
        }

        ctx.emit::<OnFalseyValueEvent>(&NoPredicateArguments(()))
            .await;
        self.backup.execute(ctx, &()).await
    }
}
