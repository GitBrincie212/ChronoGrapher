use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, payload_wrapper};
use std::ops::Deref;
use std::time::Duration;
use typed_builder::TypedBuilder;

payload_wrapper!(
    /// A simple wrapper type of [`Duration`] unable to be created from foreign code in order to prevent
    /// emissions of the [`OnTimeout`] event from other sources and keeping things encapsulated.
    ///
    /// # See Also
    /// - [`OnTimeout`] - The event which uses this wrapper as its payload.
    /// - [`TimeoutTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnTimeout`] event.
    TimeoutDuration(Duration)
);

define_event!(
    /// A [`TaskHookEvent`] triggered when a [`TimeoutTaskFrame`] times out its workflow.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`TimeoutTaskFrame`], it's the only place it is emitted
    /// after a timeout has occurred (the workflow ran for more time than the maximum amount of time configured).
    ///
    /// # Payload Type
    /// The payload type consists of only one parameter that being [`TimeoutDuration`] which is the configured
    /// maximum amount of time the workflow can run for before being timed out (can be turned into a std [`Duration`]).
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`TimeoutTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own timeouts (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`TimeoutDuration`] - The configured duration object
    /// - [`TimeoutTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`TimeoutTaskFrame`]
    OnTimeout, Duration
);

/// A trait which depends on the subtrait [`TaskError`] which allows the specification of a default timeout error
/// for the [`TimeoutTaskFrame`] as opposed to manually specifying it in its configurations.
///
/// # Required Method(s)
/// When it comes to implementing this trait, its only required method is [`DefaultTimeoutError::default_timeout_error`]
/// which is the method acts as constructor for the error.
///
/// # Required Subtrait(s)
/// [`DefaultTimeoutError`] requires as only subtrait [`TaskError`], which is for task-related errors.
///
/// # Implementation(s)
/// The [`DefaultTimeoutError`] trait is implemented for dynamic-based values such as ``String``
/// but also integrates with error-handling crates such as [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) and
/// [`eyre`](https://docs.rs/eyre/latest/eyre/) via enabling the respective feature flags.
///
/// # Object Safety / Dynamic Dispatching
/// This trait is **NOT** object safe as it constructs the type itself from "thin air" (falls in the
/// same umbrella as Rust's [`Default`] trait).
///
/// # See Also
/// - [`TaskError`] - The sub-trait which describes the errors for workflows.
/// - [`TimeoutTaskFrame`] - The [`TaskFrame`] responsible for using this trait
pub trait DefaultTimeoutError: TaskError {
    /// Constructs the error type. For more information view [`DefaultTimeoutError`].
    fn default_timeout_error() -> Self;
}

impl DefaultTimeoutError for String {
    fn default_timeout_error() -> Self {
        "Timeout Occurred".to_string()
    }
}

#[cfg(feature = "anyhow")]
impl DefaultTimeoutError for anyhow::Error {
    fn default_timeout_error() -> Self {
        anyhow::anyhow!("Timeout Occurred")
    }
}

#[cfg(feature = "eyre")]
impl DefaultTimeoutError for eyre::Error {
    fn default_timeout_error() -> Self {
        eyre::eyre!("Timeout Occurred")
    }
}

/// The [`TimeoutTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// timing out its nested [`TaskFrame`] / workflow after a certain configured amount of time has passed.
///
/// # Decorating / Wrapping Behavior
/// When wrapping [`TimeoutTaskFrame`] onto a workflow with a maximum configured duration. It runs it immediately
/// while measuring in the background the time it takes. If it completes sooner than the threshold it
/// returns anything which the workflow disposes.
///
/// On the other hand if it exceeds this threshold, it stops the workflow's process and errors out on its
/// own, no matter if the workflow were to succeed or not.
///
/// > **IMPORTANT NOTE:** Due to async Rust limitations, it is possible for the workflow to complete even if
/// > it surpasses the configured threshold for timeout if it doesn't yield. As such ensure to yield for
/// > CPU-heavy tasks to give room for a timeout.
///
/// # Execution Error(s)
/// Any kind of error may appear from the workflow if it completes sooner than the maximum configured
/// time. Otherwise, [`TimeoutTaskFrame`] will error out with its configured error or if not explicitly
/// configured (assuming the error type implements [`DefaultTimeoutError`]), it will throw the default timeout
/// error.
///
/// # Events
/// The [`TimeoutTaskFrame`] fires only one event that being [`OnTimeout`] which is emitted when the
/// workflow is timed out (ran longer than the specified configured duration). This event contains as
/// payload the maximum configured duration that being [`TimeoutDuration`] which is a thin-wrapper around
/// [`Duration`].
///
/// # Constructor(s)
/// When it comes to creating a [`TimeoutTaskFrame`], one can use the builder via [`TimeoutTaskFrame::builder`]
/// and initializing the appropriate parameters from there and then simply building it.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. as the
/// workflow primitive equivalent for [`TimeoutTaskFrame`] inside the macro is ``timeout(...)`` which accepts one
/// required argument being ``duration``, either being a function or a constant-based [`Duration`], this configures
/// the maximum time allowed for the sub-workflow.
///
/// Additionally, the timeout error can be specified as an optional parameter via ``on_timeout``. By default,
/// it assumes the error type implements [`DefaultTimeoutError`] (if not, it will error out). The parameter
/// is useful for overriding the default timeout error. For more information it's recommended to check the
/// [`workflow`](chronographer::prelude::workflow) macro itself.
///
/// Finally, you can use [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) and specify the builder
/// method [`TaskFrameBuilder::with_timeout`](chronographer::task::TaskFrameBuilder::with_timeout) for
/// supporting default timeout errors and [`TaskFrameBuilder::with_custom_timeout`](chronographer::task::TaskFrameBuilder::with_custom_timeout)
/// for overriding the error type.
///
/// # Trait Implementation(s)
/// Apart from [`TaskFrame`] which [`TimeoutTaskFrame`] implements. There is no other prominent trait
/// which it currently implements.
///
/// # Example(s)
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(timeout(5s))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
/// # let inner: TimeoutTaskFrame<MyTaskFrame> = MyTaskFrame::workflow();
/// ```
/// Wraps ``MyTaskFrame`` inside the ``timeout`` ([`TimeoutTaskFrame`]) with a configured timeout of 5 seconds to run
/// and the default timeout error being used (for String, being ``"Timeout Occurred"``). The same script can
/// be re-written in the Base API as the following:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame already from before.
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// // Timeouts after 5 seconds with the default timeout error
/// let workflow = TimeoutTaskFrame::builder()
///     .frame(MyTaskFrame)
///     .duration(Duration::from_secs(5))
///     .build();
/// # let inner: TimeoutTaskFrame<MyTaskFrame> = workflow;
/// ```
///
/// ---
///
/// When it comes to configuring the error parameter inside the timeout, it can be achieved with:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(timeout(5s, on_timeout = "My Own Error".to_string()))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
/// ```
/// Or with the base API version, the same script more or less looks as follows:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame already from before.
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// let workflow = TimeoutTaskFrame::builder()
///     .frame(MyTaskFrame)
///     .duration(Duration::from_secs(5))
///     .timeout("My Own Error".to_string())
///     .build();
/// ```
/// ---
///
/// The [`TimeoutTaskFrame`] allows for defining dynamic values as well for both the configured
/// duration but also for the error itself via their respective ``*_fn`` versions. The macro is clever
/// about this and inspects the expression to translate it into the base API with minimal change.
///
/// For the base API we can write our code as follows:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame already from before.
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// // For our case the logic is simple for demonstrative purposes.
/// let workflow = TimeoutTaskFrame::builder()
///     .frame(MyTaskFrame)
///     .with_duration(|| Duration::from_secs(5))
///     .with_timeout(|| "My Own Error".to_string())
///     .build();
/// ```
///
/// # See Also
/// - [`TimeoutTaskFrame::builder`] - The constructor / builder for configuring it in Base API.
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow primitive simply by the name of ``timeout(...)``.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`TaskFrame`] - The core trait that [`TimeoutTaskFrame`] implements and uses.
/// - [`OnTimeout`] - The event the [`TimeoutTaskFrame`] fires when a timeout is noticed.
/// - [`DefaultTimeoutError`] - A trait which provides a default error for timeout automatically.
#[derive(TypedBuilder)]
pub struct TimeoutTaskFrame<T: TaskFrame> {
    /// The builder method which sets the primary [`TaskFrame`] / workflow.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is [`TaskFrame`] which is the primary workflow.
    ///
    /// # Returns
    /// This method returns the [`TimeoutTaskFrameBuilder`] configured with the specified
    /// primary [`TakFrame`] / workflow to chain more builder methods if needed and build the [`TimeoutTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call ``.build()``
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`TimeoutTaskFrame`] uses.
    /// - [`TimeoutTaskFrame`] - The final result of the builder.
    frame: T,

    /// A builder method which sets the duration source as a function for which the [`TimeoutTaskFrame`] decides
    /// for how much to time give for the workflow to run. There is an alias builder method via
    /// [`TimeoutTaskFrameBuilder::duration`] to define a constant std [`Duration`] as source to use.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is a function that computes the std [`Duration`] which
    /// the workflow is allowed to run up to before timeout
    ///
    /// # Returns
    /// This method returns the [`TimeoutTaskFrameBuilder`] configured with the specified duration source
    /// being a function to chain more builder methods if needed and build the [`TimeoutTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call ``.build()``
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TimeoutTaskFrameBuilder::duration`] - The alias builder method for constant std [`Duration`] as source.
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`TimeoutTaskFrame`] uses.
    /// - [`TimeoutTaskFrame`] - The final result of the builder.
    #[builder(
        setter(
            prefix = "with_",
            transform = |value: impl Fn() -> Duration + Send + Sync + 'static| {
                Box::new(value) as Box<dyn Fn() -> Duration + Send + Sync + 'static>
            }
        )
    )]
    duration: Box<dyn Fn() -> Duration + Send + Sync>,

    /// A builder method which sets the timeout error source as a function for which the [`TimeoutTaskFrame`] decides
    /// what kind of error it should throw when it ultimately times out the workflow. There is an alias builder method
    /// via [`TimeoutTaskFrameBuilder::timeout`] for a constant timeout error (given the error can be cloned)
    /// as source.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is a function with returns a [`TaskError`] that is of
    /// the same type as ``T::Error`` (the primary workflow's error).
    ///
    /// # Returns
    /// This method returns the [`TimeoutTaskFrameBuilder`] configured with the specified timeout error source
    /// being a function to chain more builder methods if needed and build the [`TimeoutTaskFrame`].
    ///
    /// # Default Value
    /// Depending on whenever the error type implements the [`DefaultTimeoutError`] this property
    /// will be autofilled with the default constructor. If not, then its required.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TimeoutTaskFrameBuilder::timeout`] - An alias builder method for constant [`TaskError`] as source.
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`TimeoutTaskFrame`] uses.
    /// - [`TimeoutTaskFrame`] - The final result of the builder.
    #[builder(
        default_code = "Box::new(<T::Error as DefaultTimeoutError>::default_timeout_error)",
        default_where(T::Error: DefaultTimeoutError),
        setter(
            prefix = "with_",
            transform = |value: impl Fn() -> T::Error + Send + Sync + 'static| {
                Box::new(value) as Box<dyn Fn() -> T::Error + Send + Sync + 'static>
            }
        )
    )]
    timeout: Box<dyn Fn() -> T::Error + Send + Sync + 'static>,
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<T: TaskFrame, __frame, __with_timeout>
    TimeoutTaskFrameBuilder<T, (__frame, (), __with_timeout)>
{
    /// A builder method which sets the duration source as a constant for which the [`TimeoutTaskFrame`] decides
    /// for how much to time give for the workflow to run. There is an alias builder method via
    /// [`TimeoutTaskFrameBuilder::with_duration`] to define a function that returns std [`Duration`] as
    /// source to use.
    ///
    /// # Argument(s)
    /// The only argument this method accepts is a std [`Duration`] acting as the constant source.
    ///
    /// # Returns
    /// This method returns the [`TimeoutTaskFrameBuilder`] configured with the specified duration source
    /// being a constant std [`Duration`] to chain more builder methods if needed and build the [`TimeoutTaskFrame`].
    ///
    /// # Default Value
    /// This field has no default value, and it will result in a compile-time error if you call ``.build()``
    /// before initializing it.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TimeoutTaskFrameBuilder::with_duration`] - An alias builder method for function-based std [`Duration`] sources.
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`TimeoutTaskFrame`] uses.
    /// - [`TimeoutTaskFrame`] - The final result of the builder.
    #[allow(
        clippy::used_underscore_binding,
        clippy::no_effect_underscore_binding,
        clippy::type_complexity
    )]
    pub fn duration(
        self,
        with_duration: Duration,
    ) -> TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            (Box<dyn Fn() -> Duration + Send + Sync>,),
            __with_timeout,
        ),
    > {
        self.with_duration(move || with_duration)
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::type_complexity)]
impl<T: TaskFrame, __frame, __with_timeout>
    TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            (Box<dyn Fn() -> Duration + Send + Sync>,),
            __with_timeout,
        ),
    >
{
    #[deprecated(note = "Repeated field duration")]
    pub fn duration(
        self,
        _: TimeoutTaskFrameBuilder_Error_Repeated_field_duration,
    ) -> TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            (Box<dyn Fn() -> Duration + Send + Sync>,),
            __with_timeout,
        ),
    > {
        self
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<T: TaskFrame, __frame, __duration> TimeoutTaskFrameBuilder<T, (__frame, __duration, ())>
where
    T::Error: Clone,
{
    /// A builder method which sets the timeout error source as a constant [`TaskError`] for which the
    /// [`TimeoutTaskFrame`] decides what kind of error it should throw when it ultimately times out the workflow.
    /// There is an alias builder method via [`TimeoutTaskFrameBuilder::with_timeout`] for a function
    /// that returns a timeout error as a sources.
    ///
    /// Unlike its alias [`TimeoutTaskFrameBuilder::with_timeout`], this builder method requires the
    /// error itself to be cloneable (implement ``Clone``).
    ///
    /// # Argument(s)
    /// The only argument this method accepts is a constant [`TaskError`] that is of the same type as
    /// ``T::Error`` (the primary workflow's error) and implements ``Clone``.
    ///
    /// # Returns
    /// This method returns the [`TimeoutTaskFrameBuilder`] configured with the specified timeout error source
    /// being a constant [`TaskError`] to chain more builder methods if needed and build the [`TimeoutTaskFrame`].
    ///
    /// # Default Value
    /// Depending on whenever the error type implements the [`DefaultTimeoutError`] this property
    /// will be autofilled with the default constructor. If not, then its required.
    ///
    /// # Builder Method Chaining
    /// Trying to set this field twice will generate a compile-time error.
    ///
    /// # See Also
    /// - [`TimeoutTaskFrameBuilder::with_timeout`] - An alias builder method for function-based [`TaskError`] as source.
    /// - [`TaskFrame`] - Trait bound for the main workflow that [`TimeoutTaskFrame`] uses.
    /// - [`TimeoutTaskFrame`] - The final result of the builder.
    #[allow(
        clippy::used_underscore_binding,
        clippy::no_effect_underscore_binding,
        clippy::type_complexity
    )]
    pub fn timeout(
        self,
        value: T::Error,
    ) -> TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            __duration,
            (Box<dyn Fn() -> T::Error + Send + Sync + 'static>,),
        ),
    > {
        self.with_timeout(move || value.clone())
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::type_complexity)]
impl<T: TaskFrame, __frame, __duration>
    TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            __duration,
            (Box<dyn Fn() -> T::Error + Send + Sync + 'static>,),
        ),
    >
where
    T::Error: Clone,
{
    #[deprecated(note = "Repeated field timeout")]
    pub fn timeout(
        self,
        _: TimeoutTaskFrameBuilder_Error_Repeated_field_timeout,
    ) -> TimeoutTaskFrameBuilder<
        T,
        (
            __frame,
            __duration,
            (Box<dyn Fn() -> T::Error + Send + Sync + 'static>,),
        ),
    > {
        self
    }
}

impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let duration = (self.duration)();
        let result = tokio::time::timeout(duration, self.frame.execute(ctx, args)).await;

        if let Ok(inner) = result {
            return inner;
        }

        ctx.emit::<OnTimeout>(&TimeoutDuration(duration)).await;
        Err((self.timeout)())
    }
}
