use std::marker::PhantomData;
use std::ops::Deref;
use crate::utils::macros::define_event;
use crate::errors::TaskError;
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::TaskFrame;
use crate::task::{Debug, TaskFrameContext};
use typed_builder::TypedBuilder;

/// A super trait that depends on [`TaskError`] which allows the specification of a default dependency error
/// for the [`DependencyTaskFrame`].
///
/// # Required Method(s)
/// When it comes to implementing this trait, its only required method is [`DefaultDependencyError::default_dependency_error`]
/// which is the method acts as constructor for the error.
///
/// # Required Subtrait(s)
/// [`DefaultDependencyError`] requires as only subtrait [`TaskError`], which is for task-related errors.
///
/// # Implementation(s)
/// The [`DefaultDependencyError`] trait is implemented for dynamic-based values such as ``String``
/// but also integrates with error-handling crates such as [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) and
/// [`eyre`](https://docs.rs/eyre/latest/eyre/) via enabling the respective feature flags.
///
/// # Object Safety / Dynamic Dispatching
/// This trait is **NOT** object safe as it constructs the type itself from "thin air" (falls in the
/// same umbrella as Rust's [`Default`] trait).
///
/// # See Also
/// - [`TaskError`] - The sub-trait which describes the errors for workflows.
/// - [`DependencyTaskFrame`] - The [`TaskFrame`] responsible for using this trait
pub trait DefaultDependencyError: TaskError {
    /// Constructs the error type. For more information view [`DefaultDependencyError`].
    fn default_dependency_error() -> Self;
}

impl DefaultDependencyError for String {
    fn default_dependency_error() -> Self {
        "Unresolved Dependencies".to_string()
    }
}

#[cfg(feature = "anyhow")]
impl DefaultDependencyError for anyhow::Error {
    fn default_dependency_error() -> Self {
        anyhow::anyhow!("Unresolved Dependencies")
    }
}

#[cfg(feature = "eyre")]
impl DefaultDependencyError for eyre::Error {
    fn default_dependency_error() -> Self {
        eyre::eyre!("Unresolved Dependencies")
    }
}

/// A trait which allows specification of a custom logic to decide whenever or not [`DependencyTaskFrame`]
/// should return an error or success for any unresolved dependencies.
/// 
/// # Required Method(s)
/// The only one required method to implement is [`DependencyUnresolve::execute`] which performs
/// the actual logic of the implementor.
/// 
/// # Implementation(s)
/// There are two main implementations of this trait those being [`DependencyUnresolveFail`] which will
/// fail when the dependencies aren't resolved and [`DependencyUnresolveSkip`] which in that case just
/// silently skips the workflow.
/// 
/// # Object Safety / Dynamic Dispatching
/// This trait is object safe (dyn compatible).
/// 
/// # Generic(s)
/// The only generic is ``T``, the error type which must implement [`TaskError`] trait and is returned by the
/// method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
/// - [`DependencyUnresolveFail`] - Implements this trait by failing on unresolved dependencies.
/// - [`DependencyUnresolveSkip`] - Implements this trait by just skipping the workflow on unresolved dependencies.
/// - [`DependencyTaskFrame`] - The [`TaskFrame`] which uses this trait.
/// - [`FrameDependency`] - Represents a dependency required for the workflow.
pub trait DependencyUnresolve<T: TaskError>: Send + Sync {
    /// Performs the main logic of the implementor. For more information view [`DependencyUnresolve`].
    fn execute(&self) -> Result<(), T>;
}

/// Implementation of the [`DependencyUnresolve`] trait which fails when the dependency is unresolved.
/// The counterpart implementation for skipping the workflow when unresolved dependencies are met
/// is [`DependencyUnresolveSkip`].
/// 
/// # Constructor(s)
/// The primary way of constructing a [`DependencyUnresolveFail`] is via the [`DependencyUnresolveFail::default`]
/// which is from the [`Default`] trait implementation.
/// 
/// # Trait Implementation(s)
/// The only trait this struct is implementing is the [`DependencyUnresolve`] which allows to be executed
/// when a dependency is unresolved.
/// 
/// # Generic(s)
/// The only generic is ``T``, the error type which must implement [`TaskError`] trait and is returned by
/// the method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
/// - [`DependencyUnresolveSkip`] - The counterpart which skips the workflow when unresolved dependencies are met
/// - [`DependencyUnresolve`] - The trait that allows to be executed when a dependency is unresolved.
pub struct DependencyUnresolveFail<T: TaskError>(PhantomData<T>);

impl<T: TaskError> Default for DependencyUnresolveFail<T> {
    fn default() -> Self {
        DependencyUnresolveFail(PhantomData)
    }
}

impl<T: TaskError> Clone for DependencyUnresolveFail<T> {
    fn clone(&self) -> Self {
        DependencyUnresolveFail(PhantomData)
    }
}

impl<T: DefaultDependencyError> DependencyUnresolve<T> for DependencyUnresolveFail<T> {
    fn execute(&self) -> Result<(), T> {
        Err(T::default_dependency_error())
    }
}

/// Implementation of the [`DependencyUnresolve`] trait which silently skips the workflow when the
/// dependency is unresolved. The counterpart implementation for failing the workflow when unresolved dependencies
/// are met is [`DependencyUnresolveFail`].
/// 
/// # Constructor(s)
/// The primary way of constructing a [`DependencyUnresolveSkip`] is via the [`DependencyUnresolveSkip::default`]
/// which is from the [`Default`] trait implementation.
/// 
/// # Trait Implementation(s)
/// The only trait this struct is implementing is the [`DependencyUnresolve`] which allows to be executed
/// when a dependency is unresolved.
/// 
/// # Generic(s)
/// The only generic is ``T``, the error type which must implement [`TaskError`] trait and is returned
/// by the method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
/// - [`DependencyUnresolveFail`] - The counterpart which fails the workflow when unresolved dependencies are met
/// - [`DependencyUnresolve`] - The trait that allows to be executed when a dependency is unresolved.
pub struct DependencyUnresolveSkip<T: TaskError>(PhantomData<T>);

impl<T: TaskError> Default for DependencyUnresolveSkip<T> {
    fn default() -> Self {
        DependencyUnresolveSkip(PhantomData)
    }
}

impl<T: TaskError> Clone for DependencyUnresolveSkip<T> {
    fn clone(&self) -> Self {
        DependencyUnresolveSkip(PhantomData)
    }
}

impl<T: TaskError> DependencyUnresolve<T> for DependencyUnresolveSkip<T> {
    fn execute(&self) -> Result<(), T> {
        Ok(())
    }
}

/// A simple wrapper type of reference [`FrameDependency`] indicating the validated dependency which is unable to be created from
/// foreign code in order to prevent emissions of the [`OnDependencyValidation`] event from other sources and keeping things encapsulated.
///
/// # See Also
/// - [`IsResolved`] - Adjacent type in the [`OnDependencyValidation`] event.
/// - [`OnDependencyValidation`] - The event which uses this wrapper as its payload.
/// - [`DependencyTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnDependencyValidation`] event.
pub struct TargetDependency<'a>(&'a FrameDependency);

impl<'a> From<TargetDependency<'a>> for &'a FrameDependency {
    fn from(value: TargetDependency<'a>) -> Self {
        value.0
    }
}

impl Deref for TargetDependency<'_> {
    type Target = FrameDependency;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// A simple wrapper type of reference [`bool`] indicating whether the dependency is resolved which is unable to be created from
/// foreign code in order to prevent emissions of the [`OnDependencyValidation`] event from other sources and keeping things encapsulated.
///
/// # See Also
/// - [`TargetDependency`] - Adjacent type in the [`OnDependencyValidation`] event.
/// - [`OnDependencyValidation`] - The event which uses this wrapper as its payload.
/// - [`DependencyTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnDependencyValidation`] event.
pub struct IsResolved(bool);

impl From<IsResolved> for bool {
    fn from(value: IsResolved) -> Self {
        value.0
    }
}

impl Deref for IsResolved {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

define_event!(
    /// A [`TaskHookEvent`] triggered when a [`DependencyTaskFrame`] is validating the dependency.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`DependencyTaskFrame`], it's the only place it is emitted
    /// after a dependency has beed validated.
    ///
    /// # Payload Type
    /// The payload type consists of 2 parameters those being [`TargetDependency`] (wrapper type for `&FrameDependency`) which is the configured
    /// dependency requred to be resolved before the workflow can run and [`IsResolved`] (wrapper type for [`bool`]), which indicates whether the dependency
    /// has been resolved or not.
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`DependencyTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event if you plan to emit it.
    ///
    /// # See Also
    /// - [`TargetDependency`] - Wrapper type for [`FrameDependency`], for encapsulation reasons.
    /// - [`IsResolved`] - Wrapper type for [`bool`], for encapsulation reasons.
    /// - [`DependencyTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DependencyTaskFrame`]
    OnDependencyValidation, (TargetDependency<'a>, IsResolved)
);

/// The [`DependencyTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// dependencies ([`FrameDependency`]) of the nested [`TaskFrame`] / workflow.
/// 
/// # Decorating / Wrapping Behavior
/// It tries to resolve the dependencies first. Examples are certain [`Task(s)`] executed before,
/// an ``AtomicBool`` flag enabled, or a composition of various dependencies.
/// 
/// If the dependency is resolved, it proceeds with the nested [`TaskFrame`] / workflow. Otherwise,
/// depending on the configured behavior it will either skip or fail this workflow (by default it skips).
/// 
/// # Execution Error(s)
/// If the [`DependencyTaskFrame`] proceeds with the nested [`TaskFrame`], it can throw an error from the workflow. 
/// Otherwise, if the dependencies aren't resolved **and a custom behavior for unresolved dependency provided**, that behavior
/// can throw its own errors.
/// 
/// # Events
/// The [`DependencyTaskFrame`] fires only one event that being [`OnDependencyValidation`] which is emitted when the
/// dependencies are checked to see whether they had been resolved.
/// 
/// # Constructor(s)
/// When it comes to creating a [`DependencyTaskFrame`], one can use the builder via [`DependencyTaskFrame::builder`]
/// and initializing the appropriate parameters from there simply building it.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. As the
/// workflow primitive equivalent for [`DependencyTaskFrame`] inside the macro is ``dependency(...)``
/// which requires an argument (``dependency``) which can show up in multiple forms either.
///
/// It is recommended to check the [`workflow`](chronographer::prelude::workflow) documentation for more
/// information about the usage.
/// 
/// # Trait Implementation(s)
/// Apart from `DependencyTaskFrame` implementing the [`TaskFrame`] trait, there is no other prominent trait to note of.
/// 
/// # Example(s)
/// ```rust
/// #[task(schedule = every!(1s)))]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// #[task(schedule = every!(4s)))]
/// #[workflow(dependency(MyTask1))]
/// async fn MyTask2(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Requires ``MyTask1`` to run at least once, regardless of its result before ``MyTask2`` ever runs.
///
/// The same example in Base API:
/// ```rust
/// # #[task(schedule = every!(1s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask2(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// 
/// let workflow = DependencyTaskFrame::builder()
///     .frame(MyTask2)
///     .dependency(FrameDependency::runs(&MyTask1, 1).await)
///     .build();
/// ```
/// 
/// ---
/// 
/// ```rust
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     dependency(MyTask1(any = 3))
/// )]
/// async fn MyTask2(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Requires `MyTask1` to run at least 3 times regardless of result. Replacing ``any`` in the macro by
/// ``successes``/``failures`` will require the `MyTask1` to succeed or fail at least 3 times respectively.
///
/// The same example in Base API:
/// ```rust
/// # #[task(schedule = every!(1s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask2(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// 
/// let workflow = DependencyTaskFrame::builder()
///     .frame(MyTask2)
///     // you can replace `runs` by `successful_runs` or `failed_runs`
///     .dependency(FrameDependency::runs(&MyTask1, 3).await)
///     .build();
/// ```
/// 
/// ---
/// 
/// You can also have different types of dependencies and even combine them:
/// 
/// ```rust
/// static MY_FLAG: AtomicBool = AtomicBool::new(false);
/// 
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     dependency(
///         // You can do more complex than just returning always false
///         flag(MY_FLAG) || dynamic(|| false)
///     )
/// )]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Requires `MY_FLAG` to be true or the closure in `dynamic(...)` to return true.
/// While rewriting same code in Base API will look like:
/// ```rust
/// # static MY_FLAG: AtomicBool = AtomicBool::new(false);
/// # 
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// let flag_dep = FrameDependency::external(|| MY_FLAG.load(Ordering::Relaxed));
/// let dynamic_dep = FrameDependency::external(|| false);
/// 
/// let workflow = DependencyTaskFrame::builder()
///     .frame(MyTask1)
///     .dependency(flag_dep | dynamic_dep)
///     .build();
/// ```
/// 
/// ---
/// 
/// Finally, we can customize what happens when the dependencies aren't resolved. For example:
/// ```rust
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     dependency(
///         !MyTask5 && (MyTask1 || MyTask2 ^ MyTask4),
///         fail
///         // or
///         // custom(|_| Err(MyErrors::ServerError))
///     )
/// )]
/// async fn MyTask3(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Now instead of just skipping this workflow part with success, it errors out.
/// And of course you can customize it via `custom(...)` as in the comment.
///
/// The same example in Base API:
/// ```rust
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask3(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
/// 
/// pub struct DependencyUnresolveCustom<T: TaskError>(PhantomData<T>);
/// impl<T: DefaultDependencyError> DependencyUnresolve<T> for DependencyUnresolveFail<T> {
///     fn execute(&self) -> Result<(), T> {
///         Err(MyErrors::ServerError)
///     }
/// }
/// 
/// let workflow = DependencyTaskFrame::builder()
///     .frame(MyTask3)
///     .dependency(!FrameDependency::runs(&MyTask5, 1).await & (FrameDependency::runs(&MyTask1, 1).await | FrameDependency::runs(&MyTask2, 1).await ^ FrameDependency::runs(&MyTask4, 1).await))
///     .unresolve(DependencyUnresolveFail::default())
///     // or
///     // .unresolve(DependencyUnresolveCustom(PhantomData))
///     .build();
/// ```
///
/// # See Also
/// - [`DependencyTaskFrame::builder`] - The constructor / builder for configuring it in Base API.
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow
///   primitive simply by the name of `dependency(...)`.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`TaskFrame`] - The core trait that [`DependencyTaskFrame`] implements and uses.
/// - [`OnDependencyValidation`] - The event the [`DependencyTaskFrame`] fires when the dependency is
///   checked to see whether it had been resolved.
/// - [`TargetDependency`] - Wrapper type for [`FrameDependency`], for encapsulation reasons.
/// - [`IsResolved`] - Wrapper type for [`bool`], for encapsulation reasons.
/// - [`FrameDependency`] - Represents a dependency required for the workflow.
/// - [`DependencyUnresolve`] - A trait which provides a default behavior when the dependency isn't resolved.
/// - [`DependencyUnresolveSkip`] - The default value for the [`DependencyUnresolve`] parameter.
/// - [`DependencyUnresolveFail`] - A counterpart of [`DependencyUnresolveSkip`]
///   for failing the workflow on unresolved dependencies
#[derive(TypedBuilder)]
pub struct DependencyTaskFrame<T: TaskFrame> {
    /// The builder method which sets the primary [`TaskFrame`] / workflow.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`TaskFrame`] which is the primary workflow.
    ///
    /// # Returns
    /// This method returns the [`DependencyTaskFrameBuilder`] configured with the specified
    /// primary [`TakFrame`] / workflow to chain more builder methods if needed and build the [`DependencyTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call `.build()`
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`DependencyTaskFrame`] uses.
    /// - [`DependencyTaskFrame`] - The final result of the builder.
    frame: T,

    /// The builder method which sets a dependency required by [`DependencyTaskFrame`] to be resolved
    /// before running the [`TaskFrame`] / workflow (set via [`.frame`](DependencyTaskFrameBuilder::frame))
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`FrameDependency`] which represents a dependency
    /// required by [`DependencyTaskFrame`] to be resolved before executing the workflow.
    ///
    /// # Returns
    /// This method returns the [`DependencyTaskFrameBuilder`] configured with the specified required
    /// dependency to chain more builder methods if needed and build the [`DependencyTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call `.build()`
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`FrameDependency`] - Represents a dependency required for the workflow.
    /// - [`.frame`](DependencyTaskFrameBuilder::frame) - The builder method for configuring the workflow
    /// - [`TaskFrame`] - The trait bound that represents a workflow.
    /// - [`DependencyTaskFrame`] - The final result of the builder.
    dependency: FrameDependency,

    /// The builder method which sets an optional custom behavior / strategy dictating how
    /// [`DependencyTaskFrame`] should act (failure / success) if the [`FrameDependency`] is unresolved
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`DependencyUnresolve`] which implements the custom
    /// logic to dictate the result type from [`DependencyTaskFrame`]
    ///
    /// # Returns
    /// This method returns the [`DependencyTaskFrameBuilder`] configured with the custom specified
    /// unresolved logic to chain more builder methods if needed and build the [`DependencyTaskFrame`].
    ///
    /// # Default Value
    /// The default value of this property is `DependencyUnresolveSkip::default()` which silently skips
    /// the workflow. For more information, see [`DependencyUnresolveSkip`].
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`DependencyUnresolve`] - A trait providing a behavior for [`DependencyTaskFrame`] when the
    ///   dependency is unresolved.
    /// - [`DependencyUnresolveSkip`] - One of the implementors of the [`DependencyUnresolve`] trait and the
    ///   default used as the logic for unresolved dependencies.
    /// - [`DependencyTaskFrame`] - The final result of the builder.
    #[builder(
        default = Box::new(DependencyUnresolveSkip::<T::Error>::default()),
        setter(transform = |ts: impl DependencyUnresolve<T::Error> + 'static| Box::new(ts) as Box<dyn DependencyUnresolve<_>>)
    )]
    unresolve: Box<dyn DependencyUnresolve<T::Error>>,
}

impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let is_resolved = self.dependency.is_resolved().await;

        ctx.emit::<OnDependencyValidation>(&(TargetDependency(&self.dependency), IsResolved(is_resolved))).await;
        if !is_resolved {
            return self.unresolve.execute()
        }

        self.frame.execute(&ctx, args).await
    }
}
