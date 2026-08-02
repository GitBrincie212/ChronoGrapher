use std::marker::PhantomData;
use std::ops::Deref;
use crate::errors::TaskError;
use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::define_event;
use std::time::Duration;

/// A simple wrapper type of [`Duration`] unable to be created from foreign code in order to prevent
/// emissions of the [`OnTimeout`] event from other sources and keeping things encapsulated.
///
/// # See Also
/// - [`OnTimeout`] - The event which uses this wrapper as its payload.
/// - [`TimeoutTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnTimeout`] event.
pub struct TimeoutDuration(Duration);

impl Deref for TimeoutDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TimeoutDuration> for Duration {
    fn from(value: TimeoutDuration) -> Duration {
        value.0
    }
}

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
    /// event for your own timeouts (or anything else) if you planned to emit it.
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

#[doc(hidden)]
pub struct TimeoutMissingBuilder(());

#[doc(hidden)]
pub struct TimeoutPresentBuilder<T>(T);

/// The [`TimeoutTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// timing out its nested [`TaskFrame`] / workflow after a certain configured amount of time has passed.
///
/// # Workflow-Primitive Equivalent
/// The workflow primitive equivalent for [`TimeoutTaskFrame`] inside the [`workflow`](chronographer::prelude::workflow)
/// macro is ``timeout(...)`` which accepts one required argument being ``duration``, either being a
/// function or a constant-based [`Duration`], this configures the maximum time allowed for the sub-workflow.
///
/// Additionally, the timeout error can be specified as an optional parameter via ``on_timeout``. By default,
/// it assumes the error type implements [`DefaultTimeoutError`] (if not, it will error out). The parameter
/// is useful for overriding the default timeout error.
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
/// it surpasses the configured threshold for timeout if it doesn't yield. As such ensure to yield for
/// CPU-heavy tasks to give room for a timeout.
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
/// Wraps ``MyTaskFrame`` inside [`TimeoutTaskFrame`] with a configured timeout of 5 seconds to run
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
///     .on_timeout("My Own Error".to_string())
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
///     .duration_fn(|| Duration::from_secs(5))
///     .on_timeout_fn(|| "My Own Error".to_string())
///     .build();
/// ```
///
/// # See Also
/// - [`TimeoutTaskFrame::builder`] - The constructor / builder for configuring it in Base API.
/// - [`workflow`] - Contains an equivalent more ergonomic workflow primitive simply by the name of ``timeout(...)``.
/// - [`TaskFrame`] - The core trait that [`TimeoutTaskFrame`] implements and uses.
/// - [`OnTimeout`] - The event the [`TimeoutTaskFrame`] fires when a timeout is noticed.
/// - [`DefaultTimeoutError`] - A trait which provides a default error for timeout automatically.
pub struct TimeoutTaskFrame<T: TaskFrame> {
    frame: T,
    max_duration: Box<dyn Fn() -> Duration + Send + Sync>,
    on_timeout: Box<dyn Fn() -> T::Error + Send + Sync + 'static>,
}

pub struct TimeoutTaskFrameBuilder<T, TS, DS, ES> {
    frame: TS,
    max_duration: DS,
    on_timeout: ES,
    _marker: PhantomData<T>
}

impl<T: TaskFrame> TimeoutTaskFrame<T> {
    /// The builder constructor used as one way to configure a [`TimeoutTaskFrame`] instance. For better
    /// ergonomics and fewer boilerplate it's best to use the [`workflow`] macro. Refer on both [`workflow`]
    /// and [`TimeoutTaskFrame`] for more information about this alternative.
    ///
    /// # Returns
    /// The constructor method returns the builder for configuring the individual parameters.
    /// There are multiple builder methods inside the returned builder such as:
    /// - [`TimeoutTaskFrameBuilder::frame`] Configures the nested [`TaskFrame`] to use
    /// - [`TimeoutTaskFrameBuilder::duration`] Configures a constant duration to use for timeout
    /// - [`TimeoutTaskFrameBuilder::duration_fn`] Configures a function-based duration to use for timeout
    /// - [`TimeoutTaskFrameBuilder::on_timeout`] Configures a constant error for timeouts, optional if the error
    ///   implements the [`DefaultTimeoutError`] trait with it being its default.
    /// - [`TimeoutTaskFrameBuilder::on_timeout_fn`] Configures a function-based error for timeouts, optional
    ///   if the error implements the [`DefaultTimeoutError`] trait with it being its default.
    /// - [`TimeoutTaskFrameBuilder::build`] Converts the builder into the [`TimeoutTaskFrame`] instance
    ///
    /// # See Also
    /// - [`TimeoutTaskFrame`] - The result from building it.
    /// - [`workflow`] - An alternative more ergonomic way of constructing [`TimeoutTaskFrame`]
    pub fn builder() -> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutMissingBuilder, TimeoutMissingBuilder> {
        TimeoutTaskFrameBuilder {
            frame: TimeoutMissingBuilder(()),
            max_duration: TimeoutMissingBuilder(()),
            on_timeout: TimeoutMissingBuilder(()),
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame, D, E> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, D, E> {
    pub fn frame(self, frame: T) -> TimeoutTaskFrameBuilder<T, TimeoutPresentBuilder<T>, D, E> {
        TimeoutTaskFrameBuilder {
            frame: TimeoutPresentBuilder(frame),
            max_duration: self.max_duration,
            on_timeout: self.on_timeout,
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame, TS, ES> TimeoutTaskFrameBuilder<T, TS, TimeoutMissingBuilder, ES> {
    pub fn duration(
        self,
        duration: Duration,
    ) -> TimeoutTaskFrameBuilder<T, TS, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync>>, ES> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: TimeoutPresentBuilder(Box::new(move || duration)),
            on_timeout: self.on_timeout,
            _marker: PhantomData,
        }
    }

    pub fn duration_fn<F>(
        self,
        f: impl Fn() -> Duration + Send + Sync + 'static,
    ) -> TimeoutTaskFrameBuilder<T, TS, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync>>, ES> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: TimeoutPresentBuilder(Box::new(f) as Box<dyn Fn() -> Duration + Send + Sync>),
            on_timeout: self.on_timeout,
            _marker: PhantomData
        }
    }
}

impl<T: TaskFrame, TS, DS> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutMissingBuilder> {
    pub fn on_timeout(
        self,
        error: T::Error,
    ) -> TimeoutTaskFrameBuilder<
        T,
        TS,
        DS,
        TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync>>,
    >
    where
        T::Error: Clone + Send + Sync + 'static,
    {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: self.max_duration,
            on_timeout: TimeoutPresentBuilder(Box::new(move || error.clone())),
            _marker: PhantomData,
        }
    }

    pub fn on_timeout_fn<F>(
        self,
        f: impl Fn() -> T::Error + Send + Sync + 'static,
    ) -> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync>>> {
        TimeoutTaskFrameBuilder {
            frame: self.frame,
            max_duration: self.max_duration,
            on_timeout: TimeoutPresentBuilder(Box::new(f) as Box<dyn Fn() -> T::Error + Send + Sync>),
            _marker: PhantomData,
        }
    }
}

impl<T: TaskFrame> TimeoutTaskFrameBuilder<
    T,
    TimeoutPresentBuilder<T>,
    TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>,
    TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync + 'static>>
>
{
    pub fn build(self) -> TimeoutTaskFrame<T> {
        TimeoutTaskFrame {
            frame: self.frame.0,
            max_duration: self.max_duration.0,
            on_timeout: self.on_timeout.0,
        }
    }
}

impl<T: TaskFrame<Error: DefaultTimeoutError>> TimeoutTaskFrameBuilder<
    T,
    TimeoutPresentBuilder<T>,
    TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>,
    TimeoutMissingBuilder
>
{
    pub fn build(self) -> TimeoutTaskFrame<T> {
        TimeoutTaskFrame {
            frame: self.frame.0,
            max_duration: self.max_duration.0,
            on_timeout: Box::new(T::Error::default_timeout_error),
        }
    }
}

struct MissingTaskFrameParamError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutMissingBuilder, ES> {
    #[deprecated(note = "Missing required parameter for TaskFrame")]
    #[allow(private_interfaces)]
    pub fn build(self, _err: MissingTaskFrameParamError) -> ! {
        panic!()
    }
}

struct MissingDurationParamError;
struct SpecifiedTaskFrameError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutPresentBuilder<T>, TimeoutMissingBuilder, ES> {
    #[deprecated(note = "Missing required parameter for Duration")]
    #[allow(private_interfaces)]
    pub fn build(self, _err: MissingDurationParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for TaskFrame")]
    #[allow(private_interfaces)]
    pub fn frame(self, _err: SpecifiedTaskFrameError) -> ! {
        panic!()
    }
}

struct SpecifiedDurationParamError;
impl<T: TaskFrame, ES> TimeoutTaskFrameBuilder<T, TimeoutMissingBuilder, TimeoutPresentBuilder<Box<dyn Fn() -> Duration + Send + Sync + 'static>>, ES> {
    #[deprecated(note = "Missing required parameter for TaskFrame")]
    #[allow(private_interfaces)]
    pub fn build(self, _err: MissingTaskFrameParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for Duration")]
    #[allow(private_interfaces)]
    pub fn duration(self, _err: SpecifiedDurationParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for TaskFrame")]
    #[allow(private_interfaces)]
    pub fn duration_fn(self, _err: SpecifiedDurationParamError) -> ! {
        panic!()
    }
}

struct SpecifiedErParamError;
impl<T: TaskFrame, TS, DS> TimeoutTaskFrameBuilder<T, TS, DS, TimeoutPresentBuilder<Box<dyn Fn() -> T::Error + Send + Sync + 'static>>> {
    #[deprecated(note = "Already specified parameter for error")]
    #[allow(private_interfaces)]
    pub fn on_timeout(self, _err: SpecifiedErParamError) -> ! {
        panic!()
    }

    #[deprecated(note = "Already specified parameter for error")]
    #[allow(private_interfaces)]
    pub fn on_timeout_fn(self, _err: SpecifiedErParamError) -> ! {
        panic!()
    }
}

impl<T: TaskFrame> TaskFrame for TimeoutTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let duration = (self.max_duration)();
        let result = tokio::time::timeout(duration, self.frame.execute(ctx, &args)).await;

        if let Ok(inner) = result {
            return inner;
        }

        ctx.emit::<OnTimeout>(&duration).await;
        Err((self.on_timeout)())
    }
}