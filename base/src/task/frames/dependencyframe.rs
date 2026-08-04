use std::marker::PhantomData;
use crate::utils::macros::define_event;
use crate::errors::TaskError;
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::TaskFrame;
use crate::task::{Debug, TaskFrameContext};
use typed_builder::TypedBuilder;

/// A trait which depends on the subtrait [`TaskError`] which allows the specification of a default dependency error
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

/// A trait which allows specification of a custom handler for unresolved dependencies.
/// 
/// # Required Method(s)
/// The only one required method to implement is [`DependencyUnresolve::execute`] which performs
/// the actual logic of the handler.
/// 
/// # Implementation(s)
/// There two main implementations of this trait: [`DependencyUnresolveFail`] which will fail when the dependencies
/// aren't resolved and [`DependencyUnresolveSkip`] which in that case just silently skips the workflow.
/// 
/// # Object Safety / Dynamic Dispatching
/// This trait is object safe (dyn compatible).
/// 
/// # Generic(s)
/// The only generic is `T`, the error type which must implement [`TaskError`] trait and is returned by the
/// method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
pub trait DependencyUnresolve<T: TaskError>: Send + Sync {
    /// Performs the main logic of the handler. For more information view [`DependencyUnresolve`].
    fn execute(&self) -> Result<(), T>;
}

/// Implementation of the [`DependencyUnresolve`] trait which fails when the dependency is not resolved.
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
/// The only generic is `T`, the error type which must implement [`TaskError`] trait and is returned by the
/// method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
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

/// Implementation of the [`DependencyUnresolve`] trait which silently skips the workflow when the dependency is not resolved.
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
/// The only generic is `T`, the error type which must implement [`TaskError`] trait and is returned by the
/// method [`DependencyUnresolve::execute`].
/// 
/// # See Also
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
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

/// Represents intermediate configuration of [`DependencyTaskFrame`].
/// 
/// # Constructor(s)
/// The primary way to create this struct is by calling [`DependencyTaskFrameConfig::builder`]
/// which returns its builder.
/// 
/// # Generic(s)
/// The only generic is `T`, the type of the nested frame which must implement [`TaskFrame`] trait.
/// 
/// # See Also
/// - [`DependencyTaskFrame`] - The result of the builder of this struct.
/// - [`TaskFrame`] - The basis (its trait implementation) for the [`DependencyTaskFrame`]
/// - [`FrameDependency`] - Represents a dependency.
/// - [`DependencyUnresolve`] - A trait which provides a default behaviour when the dependency isn't resolved.
#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    frame: T,

    dependency: FrameDependency,

    #[builder(
        default = Box::new(DependencyUnresolveSkip::<T::Error>::default()),
        setter(transform = |ts: impl DependencyUnresolve<T::Error> + 'static| Box::new(ts) as Box<dyn DependencyUnresolve<_>>)
    )]
    unresolve: Box<dyn DependencyUnresolve<T::Error>>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            dependency: config.dependency,
            unresolve: config.unresolve,
        }
    }
}

define_event!(
    /// A [`TaskHookEvent`] triggered when a [`DependencyTaskFrame`] is validating the dependency.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`DependencyTaskFrame`], it's the only place it is emitted
    /// after a timeout has occurred (the workflow ran for more time than the maximum amount of time configured).
    ///
    /// # Payload Type
    /// The payload type consists of 2 parameters those being [`FrameDependency`] which is the configured
    /// dependency requred to be resolved before the workflow can run and `bool`, which indicates whether the dependency
    /// has been resolved or not.
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`DependencyTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own timeouts (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`FrameDependency`] - Represents a dependency.
    /// - [`DependencyTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DependencyTaskFrame`]
    OnDependencyValidation, (&'a FrameDependency, bool)
);

/// The [`DependencyTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// dependencies of the nested [`TaskFrame`] / workflow.
/// 
/// # Decorating / Wrapping Behavior
/// It tries to resolve the dependecies first. Examples are an `AtomicBool` flag enabled,
/// a certain [`TaskFrame`] executed before or a composition of various dependencies.
/// 
/// If the dependecy is resolved, it proceeds with the nested [`TaskFrame`].
/// Otherwise, it will skip this workflow and if provided, will execute a custom handler for unresolved dependency.
/// 
/// # Execution Error(s)
/// If the [`DependencyTaskFrame`] proceeds with the nested [`TaskFrame`], it can throw any kind of error. 
/// Otherwise if the dependencies aren't resolved **and a custom handler for unresolved dependency provided**, that handler
/// can throw its own errors.
/// 
/// # Events
/// The [`DependencyTaskFrame`] fires only one event that being [`OnDependencyValidation`] which is emitted when the
/// dependencies are checked to see whether they had been resolved.
/// 
/// # Constructor(s)
/// When it comes to creating a [`DependencyTaskFrame`], one can use the builder via [`DependencyTaskFrame::builder`]
/// and initializing the appropriate parameters from there and then simply building it.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. As the
/// workflow primitive equivalent for [`DependencyTaskFrame`] inside the macro is `dependency(...)` which accepts one
/// required argument being `dependency`, either being an `AtomicBool` flag via `flag(...)`, a task,
/// a dynamic function returning `bool` via `dynamic(...)` or a composition of previous dependencies through logical operators
/// such as `&&` (AND), `||` (OR), `!` (NOT), or `^` (XOR).
///
/// Additionally, a handler for unresolved dependency can be specified as an optional parameter which can be an immediate failure via `fail`,
/// or a custom handler via `custom(...)`.
/// 
/// # Trait Implementation(s)
/// - [`TaskFrame`]
/// 
/// # Example(s)
/// ```rust
/// #[task(schedule = every!(1s)))]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     dependency(MyTask1)
/// )]
/// async fn MyTask2(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Requires `MyTask1` to run at least once, regardless of its result.
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
/// 
/// Requires `MyTask1` to run 3 times regardless of result. Replacing `any` by 
/// `successes`/`failures`/`consecutive_successes`/`consecutive_failures` will require the `MyTask1` to
/// success 3 times/fail 3 times/success 3 consecutive times/fail consecutive 3 times, respectively.
/// 
/// ---
/// 
/// You can also have different types of dependecies and even combine them:
/// 
/// ```rust
/// static MY_FLAG: AtomicBool = AtomicBool::new(false);
/// 
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     dependency(
///         // You can do more complex than just returning always false
///         flag(MY_FLAG) || dynamic(|| false),
///         custom(|_| Err(MyErrors::ServerError))
///     )
/// )]
/// async fn MyTask3(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Requires `MY_FLAG` to be true or the closure in `dynamic(...)` to return true.
/// 
/// ---
/// 
/// Finally we can customize what happens when the dependencies aren't resolved. For example:
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
/// And of course you can costumize it via `custom(...)` as in the comment.
/// 
/// # See Also
/// - [`DependencyTaskFrame::builder`] - The constructor / builder for configuring it in Base API.
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow primitive simply by the name of ``timeout(...)``.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`TaskFrame`] - The core trait that [`DependencyTaskFrame`] implements and uses.
/// - [`OnDependencyValidation`] - The event the [`DependencyTaskFrame`] fires when the dependency is checked to see whether it had been resolved..
/// - [`FrameDependency`] - Represents a dependency.
/// - [`DependencyUnresolve`] - A trait which provides a default behaviour when the dependency isn't resolved.
pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: T,
    dependency: FrameDependency,
    unresolve: Box<dyn DependencyUnresolve<T::Error>>,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    /// The builder constructor used as one way to configure a [`DependencyTaskFrame`] instance. For better
    /// ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow) macro
    /// or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on
    /// both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`DependencyTaskFrame`] respectively for more information.
    ///
    /// # Returns
    /// The constructor method returns the builder for configuring the individual parameters.
    /// There are multiple builder methods inside the returned builder such as:
    /// - [`DependencyTaskFrameConfigBuilder::frame`] Configures the nested [`TaskFrame`] to use
    /// - [`DependencyTaskFrameConfigBuilder::dependency`] Configures a constant duration to use for timeout
    /// - [`DependencyTaskFrameConfigBuilder::unresolve`] Configures a custom handler when the dependency isn't resolved; optional.
    /// - [`DependencyTaskFrameConfigBuilder::build`] Converts the builder into the [`DependencyTaskFrame`].
    ///
    /// # See Also
    /// - [`DependencyTaskFrame`] - The result from building it.
    /// - [`DependencyTaskFrameConfig`] - Intermediate struct between [`DependencyTaskFrameConfigBuilder`] and [`DependencyTaskFrame`].
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`DependencyTaskFrame`]
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let is_resolved = self.dependency.is_resolved().await;

        ctx.emit::<OnDependencyValidation>(&(&self.dependency, is_resolved)).await;
        if !is_resolved {
            return self.unresolve.execute()
        }

        self.frame.execute(&ctx, args).await
    }
}
