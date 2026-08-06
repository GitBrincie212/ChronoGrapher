use crate::task::TaskFrame;
use crate::task::{TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group};
use std::ops::Deref;
use std::time::Duration;

/// A simple wrapper type of std [`Duration`] unable to be created from foreign code in order to prevent
/// emissions of the [`OnDelayStart`] and [`OnDelayEnd`] events from other sources and keeping
/// things encapsulated.
///
/// # See Also
/// - [`OnDelayStart`] - One of the events which uses this wrapper as its payload.
/// - [`OnDelayEnd`] - The other event which uses this wrapper as its payload.
/// - [`DelayTaskFrame`] - The [`TaskFrame`] responsible for emitting the
pub struct Delay(Duration);

impl Deref for Delay {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Delay> for Duration {
    fn from(delay: Delay) -> Self {
        delay.0
    }
}

define_event!(
    /// A [`TaskHookEvent`] triggered before the workflow gets delayed by a specific amount of time.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`DelayTaskFrame`], it's the only place it is emitted
    /// after the [`Duration`] value has been figured out and before sleeping / delaying.
    ///
    /// # Payload Type
    /// The payload type consists of only one parameter that being [`Delay`] which is the amount
    /// of time the delay is (it can be turned into a std [`Duration`]).
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`DelayTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own delays (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`Delay`] - The amount of time the workflow will sleep for.
    /// - [`OnDelayEnd`] - The counterpart of this event which runs after the delay logic.
    /// - [`DelayTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DelayTaskFrame`]
    OnDelayStart, Delay
);

define_event!(
    /// A [`TaskHookEvent`] triggered after the workflow got delayed by a specific amount of time.
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`DelayTaskFrame`], it's the only place it is emitted
    /// after the workflow has slept for the specified time.
    ///
    /// # Payload Type
    /// The payload type consists of only one parameter that being [`Delay`] which is the amount
    /// of time the delay took (it can be turned into a std [`Duration`]).
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`DelayTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own delays (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`Delay`] - The amount of time the workflow slept for.
    /// - [`OnDelayStart`] - The counterpart of this event which runs before the delay logic.
    /// - [`DelayTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DelayTaskFrame`]
    OnDelayEnd, Delay
);

define_event_group!(
    /// A closed-form [`TaskHookEvent`] group (THEG) consisting of [`OnDelayStart`] and [`OnDelayEnd`]
    /// as the events it hosts.
    ///
    /// # Common Payload Type
    /// The common payload type consists of only one parameter that being [`Delay`] which is the amount
    /// of time the delay will or has taken (it can be turned into a std [`Duration`]).
    ///
    /// # Is Emittable?
    /// Since the events are intended for only [`DelayTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event and THEG for your own delays (or anything else) if you plan to emit it.
    ///
    /// # Supported Events
    /// The events which this THEG supports are [`OnDelayStart`] and [`OnDelayEnd`] for listening to
    /// before the delay and after the delay respectively.
    ///
    /// # See Also
    /// - [`Delay`] - The amount of time the workflow will or has slept for.
    /// - [`OnDelayStart`] - A child event of the THEG which is emitted before the delay begins.
    /// - [`OnDelayEnd`] - A child event of the THEG which is emitted after the delay ended.
    /// - [`DelayTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (the subtrait) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DelayTaskFrame`]
    DelayEvents, Delay | OnDelayStart, OnDelayEnd
);

/// The [`DelayTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// delaying the [`TaskFrame`] / workflow by a specified amount of time.
///
/// # Decorating / Wrapping Behavior
/// When wrapping [`DelayTaskFrame`] onto the workflow provided a delay. It first delays by the specified
/// amount of time before running it. The final result (success or failure) is determined by the workflow.
///
/// # Execution Error(s)
/// There are no pre-defined errors that [`DelayTaskFrame`] throws, instead every error is thrown
/// by the [`TaskFrame`] / workflow itself.
///
/// # Events
/// The [`DelayTaskFrame`] fires only two events those being [`OnDelayStart`] and [`OnDelayEnd`] where
/// the former is emitted before the actual delay begins. While the latter is emitted after the delay
/// has taken place before the workflow runs.
///
/// # Constructor(s)
/// When it comes to creating a [`DelayTaskFrame`], one can use the two constructors depending
/// on the shape of the delay, [`DelayTaskFrame::new`] for constant-based and [`DelayTaskFrame::new_with`]
/// for dynamic-based.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. as the
/// workflow primitive equivalent for [`DelayTaskFrame`] inside the macro is ``delay(...)`` which
/// accepts a delay either constant or dynamic
///
/// # Trait Implementation(s)
/// Apart from [`TaskFrame`] which [`DelayTaskFrame`] implements. There is no other prominent trait
/// which it currently implements.
///
/// # Example(s)
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(delay(2s))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
/// # let inner: DelayTaskFrame<MyTaskFrame> = MyTaskFrame::workflow();
/// ```
/// Wraps ``MyTaskFrame`` inside the ``delay`` ([`DelayTaskFrame`]) with a configured constant delay
/// The same script can be re-written in the Base API as the following:
/// ```rust
/// use std::time::Duration;
/// use chronographer::prelude::*;
///
/// // Assume we have defined MyTaskFrame already like before.
/// # #[taskframe]
/// # async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
/// #    Ok(())
/// # }
///
/// let workflow = DelayTaskFrame::new(MyTaskFrame, Duration::from_secs(2))
/// # let inner: DelayTaskFrame<MyTaskFrame> = workflow;
/// ```
///
/// ---
///
/// When it comes to configuring dynamic-based delays, it's as easy as using a function:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(delay(|x| Duration::from_secs(2)))]
/// async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     Ok(())
/// }
///
/// # let inner: DelayTaskFrame<MyTaskFrame> = MyTaskFrame::workflow();
/// ```
///
/// The same version in the base API is made like so:
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
/// let workflow = DelayTaskFrame::new_with(
///     MyTaskFrame,
///     |x| Duration::from_secs(2)
/// );
/// # let inner: DelayTaskFrame<MyTaskFrame> = workflow;
/// ```
///
/// # See Also
/// - [`DelayTaskFrame::new`] - A constructor for configuring constant-based delays in Base API.
/// - [`DelayTaskFrame::new_with`] - An alternative constructor for configuring dynamic-based delays in Base API.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow primitive simply
///   by the name of ``delay(...)``.
/// - [`OnDelayStart`] - The event which fires before the actual delay takes place.
/// - [`OnDelayEnd`] - The event which fires after the actual delay takes place.
/// - [`TaskFrame`] - The core trait that [`DelayTaskFrame`] implements and uses.
pub struct DelayTaskFrame<T: TaskFrame> {
    frame: T,
    delay: Box<dyn Fn() -> Duration + Send + Sync>,
}

impl<T: TaskFrame> DelayTaskFrame<T> {
    /// A constructor method used as one way to configure a [`DelayTaskFrame`] instance with a constant
    /// delay. For better ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow)
    /// macro or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on
    /// both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`DelayTaskFrame`] respectively for more information.
    ///
    /// For dynamic-based delays it's recommended to check the [`DelayTaskFrame::new_with`] constructor.
    ///
    /// # Argument(s)
    /// This method accepts two arguments, the former being the [`TaskFrame`] / workflow itself ``frame``
    /// to delay. Whereas the latter being the amount of time to delay via ``max_duration``.
    ///
    /// # Returns
    /// The new instance with the configured [`TaskFrame`] / workflow set to ``frame`` and a constant
    /// delay of ``max_duration``.
    ///
    /// # See Also
    /// - [`DelayTaskFrame`] - The main type the constructor is building.
    /// - [`DelayTaskFrame::new_with`] - An alternative constructor for dynamic-based delays.
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`DelayTaskFrame`]
    pub fn new(frame: T, max_duration: Duration) -> Self {
        Self {
            frame,
            delay: Box::new(move || max_duration),
        }
    }

    /// A constructor method used as one way to configure a [`DelayTaskFrame`] instance with a dynamic
    /// delay. For better ergonomics and fewer boilerplate it's best to use the [`workflow`](chronographer::prelude::workflow)
    /// macro or by utilizing the [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder). Refer on
    /// both [`workflow`](chronographer::prelude::workflow), [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder)
    /// and [`DelayTaskFrame`] respectively for more information.
    ///
    /// For constant-based delays it's recommended to check the [`DelayTaskFrame::new`] constructor.
    ///
    /// # Argument(s)
    /// This method accepts two arguments, the former being the [`TaskFrame`] / workflow itself ``frame``
    /// to delay. Whereas the latter being a function that returns the amount of time to delay via ``function``.
    ///
    /// # Returns
    /// The new instance with the configured [`TaskFrame`] / workflow set to ``frame`` and a dynamic
    /// delay of ``function``.
    ///
    /// # See Also
    /// - [`DelayTaskFrame`] - The main type the constructor is building.
    /// - [`DelayTaskFrame::new`] - An alternative constructor for constant-based delays.
    /// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
    /// - [`workflow`](chronographer::prelude::workflow) - An alternative more ergonomic way of constructing [`DelayTaskFrame`]
    pub fn new_with(frame: T, function: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            frame,
            delay: Box::new(function),
        }
    }
}

impl<T: TaskFrame> TaskFrame for DelayTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let delay = (self.delay)();

        ctx.emit::<OnDelayStart>(&Delay(delay)).await;
        tokio::time::sleep(delay).await;
        ctx.emit::<OnDelayEnd>(&Delay(delay)).await;

        self.frame.execute(ctx, args).await
    }
}
