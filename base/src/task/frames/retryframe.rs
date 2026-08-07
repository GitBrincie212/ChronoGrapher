use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group, payload_wrapper};
use std::clone::Clone;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::time::Duration;
use typed_builder::TypedBuilder;

/// A trait acting as a filter (predicate) for [`RetriableTaskFrame`] for deciding whenever or not to retry
/// an error (or to halt the retrying process and return the error).
///
/// # Required Method(s)
/// When it comes to implementing this trait, its only required method is [`RetryErrorFilter::filter`]
/// which is the method acting as the predicate for the filtering.
///
/// # Implementation(s)
/// The [`RetryErrorFilter`] trait is implemented for function pointers accepting an error parameter of ``T``
/// and return a boolean. While also for boolean values (which when filtering they always return that value)
///
/// # Object Safety / Dynamic Dispatching
/// This trait is object safe (dyn compatible).
///
/// # Generic(s)
/// The only generic is ``T``, the error type which must implement [`TaskError`] trait which is filtered
/// against to check whenever to proceed or to retry.
///
/// # See Also
/// - [`RetriableTaskFrame`] - The main intended usage of this trait
/// - [`TaskError`] - The trait of the generic which describes the errors for workflows.
pub trait RetryErrorFilter<T: TaskError>: Send + Sync + 'static {
    /// Performs the filtering logic, see [`RetryErrorFilter`] for more info.
    fn filter(&self, error: &T) -> bool;
}

impl<T: TaskError> RetryErrorFilter<T> for fn(&T) -> bool {
    fn filter(&self, error: &T) -> bool {
        self(error)
    }
}

impl<T: TaskError> RetryErrorFilter<T> for bool {
    fn filter(&self, _error: &T) -> bool {
        *self
    }
}

/// A trait which computes the amount of time to delay given the number of retry for [`RetriableTaskFrame`].
///
/// # Required Method(s)
/// When it comes to implementing this trait, its only required method is [`RetryBackoffStrategy::compute`]
/// which is the method computing the actual delay.
///
/// # Implementation(s)
/// There are many built-in implementations of this trait present throughout ChronoGrapher:
/// - [`ConstantBackoffStrategy`] - Returns a constant amount of delay untied to the retry
/// - [`LinearBackoffStrategyConfig`] - Based on a start and a growth factor, returns a
/// delay growing proportional to its retry count linearly.
/// - [`ExponentialBackoffStrategy`] - Based on a factor and a ceiling, returns a
/// delay growing proportional to its retry count exponentially.
/// - [`JitterBackoffStrategy`] - Based on a [`RetryBackoffStrategy`] and a jitter type, it
/// computes the delay and randomizes it based on the jitter type.
///
/// # Object Safety / Dynamic Dispatching
/// This trait is object safe (dyn compatible).
///
/// # See Also
/// - [`ConstantBackoffStrategy`] - An implementation for constant delays
/// - [`LinearBackoffStrategyConfig`] - An implementation for linear delays
/// - [`ExponentialBackoffStrategy`] - An implementation for exponential delays
/// - [`JitterBackoffStrategy`] - An implementation for jittered delays
/// - [`RetriableTaskFrame`] - The main intended usage of this trait
pub trait RetryBackoffStrategy: Send + Sync + 'static {
    /// Performs the actual computing for the delay, accepting the retry count and returning
    /// a [`Duration`] representing the delay back. Inspect [`RetryBackoffStrategy`] for more info.
    fn compute(&self, retry: u32) -> Duration;
}

/// An implementation of [`RetryBackoffStrategy`] which returns the same delay it was given
/// at construction time no matter the retry.
///
/// # Constructor(s)
/// The only way to construct a [`ConstantBackoffStrategy`] is via [`ConstantBackoffStrategy::new`]
/// method which accepts a constant duration to use for computing.
///
/// # Trait Implementation(s)
/// Apart from implementing the [`RetryBackoffStrategy`]. There are no other prominent traits it
/// currently implements.
///
/// # See Also
/// - [`RetryBackoffStrategy`] - The main trait this component implements.
/// - [`LinearBackoffStrategyConfig`] - An alternative implementation for linear delays.
/// - [`ExponentialBackoffStrategy`] - An alternative implementation for exponential delays.
/// - [`JitterBackoffStrategy`] - An alternative implementation for jittered delays.
#[repr(transparent)]
pub struct ConstantBackoffStrategy(Duration);

impl ConstantBackoffStrategy {
    /// The constructor method for [`ConstantBackoffStrategy`] which accepts a std [`Duration`]
    /// and uses it for its delay computation regardless of retry count.
    ///
    /// # Argument(s)
    /// The method accepts one argument, that being ``duration`` of type std [`Duration`] which is
    /// the amount of time of the delay. Providing a [`Duration::ZERO`] will result in no-delay.
    ///
    /// # Returns
    /// The constructed [`ConstantBackoffStrategy`] object with the configured delay being as long
    /// as the ``duration`` specified parameter.
    ///
    /// # See Also
    /// - [`ConstantBackoffStrategy`] - The main component being constructed.
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl RetryBackoffStrategy for ConstantBackoffStrategy {
    fn compute(&self, _retry: u32) -> Duration {
        self.0
    }
}

/// An implementation of [`RetryBackoffStrategy`] which computes the delay exponentially based on
/// the specified parameters.
///
/// # Constructor(s)
/// The first way to construct a [`ExponentialBackoffStrategy`] is via [`ExponentialBackoffStrategy::new`]
/// method which accepts a factor (the exponent) dictating the speed at which the delay grows exponentially.
///
/// Whereas the second way involves [`ExponentialBackoffStrategy::new_with`] which behaves almost identically
/// to the former method but requires an upper bound for the delay.
///
/// # Trait Implementation(s)
/// Apart from implementing the [`RetryBackoffStrategy`]. There are no other prominent traits it
/// currently implements.
///
/// # See Also
/// - [`RetryBackoffStrategy`] - The main trait this component implements.
/// - [`LinearBackoffStrategyConfig`] - An alternative implementation for linear delays.
/// - [`ConstantBackoffStrategy`] - An alternative implementation for constant delays.
/// - [`JitterBackoffStrategy`] - An alternative implementation for jittered delays.
pub struct ExponentialBackoffStrategy(f64, f64);

impl ExponentialBackoffStrategy {
    /// A constructor method for [`ExponentialBackoffStrategy`] which only accepts an exponential
    /// factor of growth and has no upper bound. To specify an upper bound it's recommended to check
    /// [`ExponentialBackoffStrategy::new_with`]
    ///
    /// # Argument(s)
    /// The method accepts one argument, that being ``factor`` (or exponent) dictating the speed at which
    /// the exponential delay function grows.
    ///
    /// # Returns
    /// The constructed [`ExponentialBackoffStrategy`] object with the configured factor and no upper bound
    ///
    /// # See Also
    /// - [`ExponentialBackoffStrategy`] - The main component being constructed.
    /// - [`ExponentialBackoffStrategy::new_with`] - An alternative constructor for an upper-bounded exponential.
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn new(factor: f64) -> Self {
        Self(factor, f64::INFINITY)
    }

    /// A constructor method for [`ExponentialBackoffStrategy`] which accepts an exponential
    /// factor of growth but also an upper bound. To not specify an upper bound it's recommended to check
    /// [`ExponentialBackoffStrategy::new`]
    ///
    /// # Argument(s)
    /// The method accepts two argument, the former being ``factor`` (or exponent) dictating the speed at which
    /// the exponential delay function grows. Whereas the latter is
    ///
    /// # Returns
    /// The constructed [`ExponentialBackoffStrategy`] object with the configured factor and no upper bound
    ///
    /// # See Also
    /// - [`ExponentialBackoffStrategy`] - The main component being constructed.
    /// - [`ExponentialBackoffStrategy::new_with`] - An alternative constructor for upper-bounded exponential.
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn new_with(factor: f64, max_duration: Duration) -> Self {
        Self(factor, max_duration.as_secs_f64())
    }
}

impl RetryBackoffStrategy for ExponentialBackoffStrategy {
    fn compute(&self, retry: u32) -> Duration {
        Duration::from_secs_f64(self.0.powf(retry as f64).min(self.1))
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = LinearBackoffStrategy))]
pub struct LinearBackoffStrategyConfig {
    factor: Duration,

    #[builder(default = Duration::ZERO)]
    start: Duration,

    #[builder(default, setter(strip_option))]
    clamp: Option<Duration>,
}

impl From<LinearBackoffStrategyConfig> for LinearBackoffStrategy {
    fn from(value: LinearBackoffStrategyConfig) -> Self {
        let start = value.start.as_secs_f64();
        let factor = value.factor.as_secs_f64();
        let clamp = value
            .clamp
            .map(|x| x.as_secs_f64())
            .unwrap_or(f64::INFINITY);

        LinearBackoffStrategy {
            start,
            factor,
            clamp,
        }
    }
}

/// An implementation of [`RetryBackoffStrategy`] which computes the delay linearly based on
/// the specified parameters.
///
/// # Constructor(s)
/// The primary way of constructing a [`LinearBackoffStrategy`] is via its builder ([`LinearBackoffStrategy::builder`])
/// and specifying from there the necessary parameters and finally build it.
///
/// # Trait Implementation(s)
/// Apart from implementing the [`RetryBackoffStrategy`]. There are no other prominent traits it
/// currently implements.
///
/// # See Also
/// - [`RetryBackoffStrategy`] - The main trait this component implements.
/// - [`ConstantBackoffStrategy`] - An alternative implementation for constant delays.
/// - [`ExponentialBackoffStrategy`] - An alternative implementation for exponential delays.
/// - [`JitterBackoffStrategy`] - An alternative implementation for jittered delays.
pub struct LinearBackoffStrategy {
    start: f64,
    factor: f64,
    clamp: f64,
}

impl LinearBackoffStrategy {
    pub fn builder() -> LinearBackoffStrategyConfigBuilder {
        LinearBackoffStrategyConfig::builder()
    }
}

impl RetryBackoffStrategy for LinearBackoffStrategy {
    fn compute(&self, retry: u32) -> Duration {
        Duration::from_secs_f64((self.start * (retry as f64) * self.factor).min(self.clamp))
    }
}

enum JitterType {
    Full,
    Equal,
    Decorrelated(f64),
}

/*
    TODO: Optimizing & Refining The Current Implementation For JitterBackoffStrategy

    It may be better for performance to separate the jitter types to their own backoff strategies,
    or a global JitterBackoffStrategy with another generic for the type with each implementing their
     own logic. This would avoid a branch (switch) since the contents are inlined directly.

     Additionally, more parameters could be introduced for a greater level of control such as specifying
     a minimum bound on decorrelated jitter.
*/

/// An implementation of [`RetryBackoffStrategy`] which unlike all other built-in [`RetryBackoffStrategy`]
/// nests itself a [`RetryBackoffStrategy`] and based on what it computes, uses the result to randomize
/// it based on a few parameters, returning that back.
///
/// There are three jitter types each with their own behavior:
/// - **Full Jitter:** Generates a random delay from zero to the amount of time calculated.
/// - **Equal Jitter:** Generates a random delay between half of the amount of time to the amount of time
/// - **Decorrelated Jitter** Generates a random delay based on previous delays.
///
/// For more information, one can read this article about the three jitter types and where they're used:
/// https://dev.to/rafael_panisset/retry-strategies-compared-constant-vs-exponential-backoff-vs-jitter-in-go-with-simulation-1mce
///
/// > **NOTE:** While the concepts are in Golang, they do transfer over to the Rust world.
///
/// # Constructor(s)
/// Each jitter type has its own constructor, all requiring a factor parameter that intensifies the effect.
/// - [`JitterBackoffStrategy::full`] - For "Full Jitter"
/// - [`JitterBackoffStrategy::equal`] - For "Equal Jitter"
/// - [`JitterBackoffStrategy::decorrelated`] - For "Decorrelated Jitter", requiring its own parameter.
///
/// # Trait Implementation(s)
/// Apart from implementing the [`RetryBackoffStrategy`]. There are no other prominent traits it
/// currently implements.
///
/// # See Also
/// - [`RetryBackoffStrategy`] - The main trait this component implements.
/// - [`ConstantBackoffStrategy`] - An alternative implementation for constant delays.
/// - [`LinearBackoffStrategy`] - An alternative implementation for jittered delays.
/// - [`ExponentialBackoffStrategy`] - An alternative implementation for exponential delays.
pub struct JitterBackoffStrategy<T: RetryBackoffStrategy> {
    backoff: T,
    factor: f64,
    jitter_type: JitterType,
}

impl<T: RetryBackoffStrategy> JitterBackoffStrategy<T> {
    /// A constructor method for **Full-Jittered** [`JitterBackoffStrategy`]. Full jitter types generate
    /// a random delay between zero and the amount of time. For more information view [`JitterBackoffStrategy`].
    ///
    /// # Argument(s)
    /// The method accepts one argument, that being ``factor`` dictating the effect of this
    /// randomization / jitter that takes place.
    ///
    /// # Returns
    /// The constructed **Full-Jittered** [`JitterBackoffStrategy`] object with the configured ``factor``.
    ///
    /// # See Also
    /// - [`JitterBackoffStrategy`] - The main component being constructed.
    /// - [`JitterBackoffStrategy::equal`] - An alternative constructor for Equal-Jitter [`JitterBackoffStrategy`].
    /// - [`JitterBackoffStrategy::decorrelated`] - An alternative constructor for Decorrelated-Jitter [`JitterBackoffStrategy`].
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn full(strat: T, factor: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::Full,
        }
    }

    /// A constructor method for **Equal-Jittered** [`JitterBackoffStrategy`]. Equal jitter types generate
    /// a random delay between the half the amount of time to the full amount of time. For more information
    /// view [`JitterBackoffStrategy`].
    ///
    /// # Argument(s)
    /// The method accepts one argument, that being ``factor`` dictating the effect of this
    /// randomization / jitter that takes place.
    ///
    /// # Returns
    /// The constructed **Equal-Jittered** [`JitterBackoffStrategy`] object with the configured ``factor``.
    ///
    /// # See Also
    /// - [`JitterBackoffStrategy`] - The main component being constructed.
    /// - [`JitterBackoffStrategy::full`] - An alternative constructor for Full-Jitter [`JitterBackoffStrategy`].
    /// - [`JitterBackoffStrategy::decorrelated`] - An alternative constructor for Decorrelated-Jitter [`JitterBackoffStrategy`].
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn equal(strat: T, factor: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::Equal,
        }
    }

    /// A constructor method for **Decorrelated-Jittered** [`JitterBackoffStrategy`]. Decorrelated jitter
    /// types unlike the other two operate on the previous calculated delay and require an extra parameter
    /// specifying the upper bound. For more information view [`JitterBackoffStrategy`].
    ///
    /// # Argument(s)
    /// The method accepts two arguments where the former is the ``factor`` dictating the effect of this
    /// randomization / jitter that takes place. Whereas the latter being ``max`` is an upper-bound
    /// specifically for the decorrelated jitter.
    ///
    /// # Returns
    /// The constructed **Equal-Jittered** [`JitterBackoffStrategy`] object with the configured ``factor``
    /// plus an upper bound being equal to ``max``.
    ///
    /// # See Also
    /// - [`JitterBackoffStrategy`] - The main component being constructed.
    /// - [`JitterBackoffStrategy::full`] - An alternative constructor for Full-Jitter [`JitterBackoffStrategy`].
    /// - [`JitterBackoffStrategy::equal`] - An alternative constructor for Equal-Jitter [`JitterBackoffStrategy`].
    /// - [`RetryBackoffStrategy`] - The main trait for computing delays.
    pub const fn decorrelated(strat: T, factor: f64, max: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::Decorrelated(max),
        }
    }
}

impl<T: RetryBackoffStrategy> RetryBackoffStrategy for JitterBackoffStrategy<T> {
    fn compute(&self, retry: u32) -> Duration {
        let base = self.backoff.compute(retry).mul_f64(self.factor);

        let base_secs = base.as_secs_f64();

        let secs = match self.jitter_type {
            JitterType::Full => fastrand::f64() * base_secs,

            JitterType::Equal => {
                let half = base_secs / 2.0;
                half + (fastrand::f64() * half)
            }

            JitterType::Decorrelated(max) => {
                // TODO: This is an approximation, might get fixed in the future
                let upper = (base_secs * 3.0).min(max);

                fastrand::f64() * upper
            }
        };

        Duration::from_secs_f64(secs)
    }
}

payload_wrapper!(
    /// A simple wrapper type of ``u32`` unable to be created from foreign code in order to prevent
    /// emissions of the [`OnRetryAttemptStart`] and [`OnRetryAttemptEnd`] events from other sources and
    /// keeping things encapsulated.
    ///
    /// # See Also
    /// - [`OnRetryAttemptStart`] - One of the events which uses this wrapper as its payload.
    /// - [`OnRetryAttemptEnd`] - One of the events which uses this wrapper in its payload.
    /// - [`RetriableTaskFrame`] - The [`TaskFrame`] responsible for emitting
    ///   the [`OnRetryAttemptStart`] and [`OnRetryAttemptStart`] events.
    RetryCount(u32)
);

payload_wrapper!(
    /// A simple wrapper type of option reference of [`TaskError`] unable to be created from foreign code in order to
    /// prevent emissions of the [`OnRetryAttemptEnd`] event from other sources and keeping things encapsulated.
    ///
    /// # See Also
    /// - [`OnRetryAttemptEnd`] - The event which uses this wrapper as its payload.
    /// - [`TimeoutTaskFrame`] - The [`TaskFrame`] responsible for emitting the [`OnRetryAttemptEnd`] event.
    RetryError<'a>(Option<&'a dyn TaskError>)
);

define_event!(
    /// A [`TaskHookEvent`] triggered before the workflow gets retried (or attempted for the first time).
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`RetriableTaskFrame`], it's the only place it is emitted
    /// after the (initial) retry takes place and before the workflow runs.
    ///
    /// # Payload Type
    /// The payload type consists of only one parameter that being [`RetryCount`] which is the total
    /// number of retries attempted (with a value of zero, indicating it's the first-time trying the
    /// workflow and no past errors have occurred).
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`RetriableTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own retries (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`RetryCount`] - The number of retries that have occurred.
    /// - [`OnRetryAttemptEnd`] - The counterpart of this event which runs after the workflow runs.
    /// - [`RetriableTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`RetriableTaskFrame`].
    OnRetryAttemptStart, RetryCount
);

define_event!(
    /// A [`TaskHookEvent`] triggered after the workflow gets retried (or attempted for the first time).
    ///
    /// # Sources Of Emission
    /// Since the event is primarily concerned with [`RetriableTaskFrame`], it's the only place it is emitted
    /// after the (initial) retry takes place and before the workflow runs.
    ///
    /// # Payload Type
    /// The payload type consists of two parameters, the former being [`RetryCount`] which is the total
    /// number of retries attempted (with a value of zero, indicating it's the first-time trying the
    /// workflow and no past errors have occurred).
    ///
    /// Whereas the latter being [`RetryError`], an optional error that indicates any failure (error)
    /// that happened from the workflow itself.
    ///
    /// # Is Emittable?
    /// Since the event is intended for only [`RetriableTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event for your own retries (or anything else) if you plan to emit it.
    ///
    /// # See Also
    /// - [`RetryCount`] - The number of retries that have occurred.
    /// - [`OnRetryAttemptStart`] - The counterpart of this event which runs before the workflow runs.
    /// - [`RetriableTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (its trait implementation) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`RetriableTaskFrame`].
    OnRetryAttemptEnd, (RetryCount, RetryError<'a>)
);

define_event_group!(
    /// A closed-form [`TaskHookEvent`] group (THEG) consisting of [`OnRetryAttemptStart`] and
    /// [`OnRetryAttemptEnd`] as the events it hosts.
    ///
    /// # Common Payload Type
    /// While the events themselves have an overlapping payload type being [`RetryCount`]. They
    /// can't contain it as a common payload type
    ///
    /// # Is Emittable?
    /// Since the events are intended for only [`RetriableTaskFrame`], the event is **NOT** emittable from
    /// outside code (primarily for encapsulation reasons). It's recommended to either make your own
    /// event and THEG for your own retries (or anything else) if you plan to emit it.
    ///
    /// # Supported Events
    /// The events which this THEG supports are [`OnRetryAttemptEnd`] and [`OnRetryAttemptEnd`] for
    /// listening to before the retry takes place and after the retry took place respectively.
    ///
    /// # See Also
    /// - [`Delay`] - The amount of time the workflow will or has slept for.
    /// - [`OnRetryAttemptStart`] - A child event of the THEG which is emitted before the retry begins.
    /// - [`OnRetryAttemptEnd`] - A child event of the THEG which is emitted after the retry ended.
    /// - [`RetriableTaskFrame`] - The [`TaskFrame`] responsible for emitting the event.
    /// - [`TaskHookEvent`] - The basis (the subtrait) for this event.
    /// - [`TaskFrame`] - The basis (its trait implementation) for the [`DelayTaskFrame`]
    RetryAttemptEvents, OnRetryAttemptStart, OnRetryAttemptEnd
);

/// The [`RetriableTaskFrame`] is a wrapper-based / decorator [`TaskFrame`] (workflow primitive) which handles
/// the retry logic of the nested [`TaskFrame`] / workflow.
///
/// # Decorating / Wrapping Behavior
/// Initially it attempts to execute the [`TaskFrame`] / workflow as if it didn't exist. If it fails,
/// it proceeds with an error filter ([`RetryErrorFilter`]) to decide whenever or not to re-attempt
/// the error.
///
/// If yes then it computes via a backoff strategy ([`RetryBackoffStrategy`]) a delay to sleep for (can be
/// zero to omit this process), after sleeping the [`RetriableTaskFrame`] retries again following the same
/// algorithm.
///
/// If the answer was no however, it will return the error back immediately without any other retries
/// involved. In either case if the retries are exhausted or the filter decides to filter out the error,
/// it will return the error from the workflow.
///
/// > **NOTE:** A limitation of [`RetriableTaskFrame`] is it doesn't keep a history of its errors for
/// performance reasons, a work-around is to attach a [`TaskHook`] in the [`OnRetryAttemptEnd`] event.
///
/// # Execution Error(s)
/// There are no pre-defined errors that [`RetriableTaskFrame`] throws, instead every error is thrown
/// by the [`TaskFrame`] / workflow itself.
///
/// # Events
/// The [`RetriableTaskFrame`] fires two events those being [`OnRetryAttemptStart`] and [`OnRetryAttemptEnd`].
/// The former is emitted when a (initial) retry begins and before the workflow runs. Whereas the latter is
/// emitted after the workflow has been (initially) retried with its results available.
///
/// # Constructor(s)
/// When it comes to creating a [`RetriableTaskFrame`], one can use the builder via [`RetriableTaskFrame::builder`]
/// and initializing the appropriate parameters from there simply building it.
///
/// Another way to achieve this is via the [`workflow`](chronographer::prelude::workflow) macro. As the
/// workflow primitive equivalent for [`RetriableTaskFrame`] inside the macro is ``retry(...)``.
/// It is recommended to check the [`workflow`](chronographer::prelude::workflow) documentation for more
/// information about the usage.
///
/// # Trait Implementation(s)
/// Apart from [`RetriableTaskFrame`] implementing the [`TaskFrame`] trait, there are no other prominent
/// traits to note of.
///
/// # Example(s)
/// ```rust
/// use chronographer::prelude::*;
///
/// #[task(schedule = every!(3s)))]
/// #[workflow(retry(5))]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Initially tries ``MyTask1`` and if successful it returns immediately, but if a failure occurs and
/// isn't filtered out then it immediately attempts again the algorithm. This process continues until
/// either the retry count has been exhausted, or success is encountered or a failure which is filtered out.
///
/// The same example in Base API:
/// ```rust
/// use chronographer::prelude::*;
/// use std::num::NonZeroU32;
///
/// # #[task(schedule = every!(3s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
///
/// let workflow = RetriableTaskFrame::builder()
///     .frame(MyTask1)
///     .retries(NonZeroU32::new(5).unwrap())
///     .build();
/// ```
///
/// ---
///
/// Though when it comes to instant retries, they aren't either particularly useful (only in some niche
/// scenarios). Which is why we can customize the delay in-between:
///
/// ```rust
/// use chronographer::prelude::*;
///
/// #[task(schedule = every!(3s)))]
/// #[workflow( retry(5, delay = 500ms))]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// This is a modified example of the previous example, the main change in behavior is now each retry
/// will have a delay in-between instead of immediately retrying.
///
/// Modifying example, this can be adapted in Base API like so:
/// ```rust
/// use std::time::Duration;
/// use std::num::NonZeroU32;
/// use chronographer::prelude::*;
///
/// # #[task(schedule = every!(3s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
///
/// let workflow = RetriableTaskFrame::builder()
///     .frame(MyTask1)
///     .retries(NonZeroU32::new(5).unwrap())
///     .constant(Duration::from_millis(500))
///     .build();
/// ```
///
/// ---
///
/// In the above example, it's still not enough in some cases. Retry delays in the real-world it requires
/// some form of growth in proportion with the number of retries that have occurred (since the error isn't
/// resolved).
///
/// There are multiple ways of achieving this via [`RetryBackoffStrategy`] from the most basic we've
/// seen [`ConstantBackoffStrategy`] for constant-based delays, [`LinearBackoffStrategy`] for
/// linear-based delays, [`ExponentialBackoffStrategy`] for exponential-based delays to [`JitterBackoffStrategy`]
/// for randomized delays via a nested backoff strategy.
///
/// ```rust
/// use chronographer::prelude::*;
///
/// #[task(schedule = every!(3s)))]
/// #[workflow(
///     // These retries are permutations (not meant to be applied one after the other)
///     retry(5, delay = constant(500ms)) // Identical to without constant(...)
///     retry(5, delay = linear(50ms)) // Linear-based grows by 50ms for every retry
///     retry(5, delay = exponential(2.0)) // Exponential-based grows by delay^2 for every retry
///     retry(5, delay = jitter(full, constant(500ms), 2)) // Full-Jitter based (randomized) delay
/// )]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
///
/// For more information it's recommended to check the macro itself as well as the respective documentation
/// of each [`RetryBackoffStrategy`] to learn about their behavior. However, in short, the first expression
/// is an alternative way of expressing constant-based delays.
///
/// The second expression declares a linear-based backoff strategy which grows by a factor of 50 milliseconds
/// for every retry (the lower and upper bound can also be configured).
///
/// Whereas the third expression declares an exponential-based backoff strategy growing by ``f^n``
/// where ``f`` is our factor and ``n`` is the number of retries. An upper bound can also be configured
/// in order for it to not grow infinitely.
///
/// Lastly the fourth expression declares a jitter-based backoff strategy configured to be of type
/// ``full`` and generates a delay from the computed value of ``constant(500ms)`` (or just ``500ms``)
/// from zero to the computed value.
///
/// The same example with these permutations in Base API can be achieved as follows:
/// ```rust
/// use std::time::Duration;
/// use std::num::NonZeroU32;
/// use chronographer::prelude::*;
///
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
///
/// // Same as before the builders for backoffs are meant to be viewed isolated
/// let workflow = RetriableTaskFrame::builder()
///     .frame(MyTask1)
///     .retries(NonZeroU32::new(5).unwrap())
///     .constant(Duration::from_millis(500))
///     .linear(Duration::from_millis(50))
///     .exponential(2.0)
///     .full_jitter(
///         ConstantBackoffStrategy::new(Duration::from_millis(500)),
///         5.0
///     )
///     .build();
/// ```
///
/// ---
///
/// Finally, the above examples blindly retry every kind of error with no inspection. Most
/// application-specific errors that stem from bugs don't require retrying as they result in the same
/// error. Instead, you can filter these errors:
///
/// ```rust
/// #[task(schedule = every!(4s)))]
/// #[workflow(
///     retry(5, 500ms, when = [MyErrors::VariantA, MyErrors::VariantB(1)])
/// )]
/// async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
///     // ...
/// }
/// ```
/// Now every error that follow either patterns of ``MyErrors::VariantA`` and ``MyErrors::VariantB(1)``
/// **WILL** be retried, every other error will be rejected. It utilizes Rust's pattern-matching syntax
/// which more info can be viewed in the [`workflow`](chronographer::prelude::workflow) macro itself.
///
/// While rewriting same code in Base API will look like:
/// ```rust
/// use std::time::Duration;
/// use std::num::NonZeroU32;
/// use chronographer::prelude::*;
///
/// # #[task(schedule = every!(4s)))]
/// # async fn MyTask1(ctx: &TaskFrameContext) -> Result<(), MyErrors> {
/// #     // ...
/// # }
///
/// // Or can be manually implemented via the trait
/// fn my_error_filter(error: &MyErrors) -> bool {
///     matches!(error, MyErrors::VariantA) ||
///         matches!(error, MyErrors::VariantB(1))
/// }
///
/// let workflow = RetriableTaskFrame::builder()
///     .frame(MyTask1)
///     .retries(NonZeroU32::new(5).unwrap())
///     .constant(Duration::from_millis(500))
///     .filter(my_error_filter)
///     .build();
/// ```
///
/// # See Also
/// - [`RetriableTaskFrame::builder`] - The constructor / builder for configuring it in Base API.
/// - [`workflow`](chronographer::prelude::workflow) - Contains an equivalent more ergonomic workflow
///   primitive simply by the name of ``retry(...)``.
/// - [`TaskFrameBuilder`](chronographer::task::TaskFrameBuilder) A middle-ground between the macro and the base API
/// - [`RetryBackoffStrategy`] - The computation unit that calculates the delay given a retry.
/// - [`RetryErrorFilter`] - The filtering unit for deciding which errors to retry and which not.
/// - [`TaskFrame`] - The core trait that [`RetriableTaskFrame`] implements and uses.
/// - [`OnRetryAttemptStart`] - An event the [`RetriableTaskFrame`] fires when the (initial) retry is attempted.
/// - [`OnRetryAttemptEnd`] - An event the [`RetriableTaskFrame`] fires when the (initial) retry has finished.
/// - [`RetryCount`] - Wrapper type for ``u32``, for encapsulation reasons.
/// - [`RetryError`] - Wrapper type for an option of reference [`TaskError`], for encapsulation reasons.
/// - [`ConstantBackoffStrategy`] - A basic implementation of [`RetryBackoffStrategy`] for constant-based delays.
/// - [`LinearBackoffStrategy`] - An implementation of [`RetryBackoffStrategy`] for linear-based delays.
/// - [`ExponentialBackoffStrategy`] - An implementation of [`RetryBackoffStrategy`] for exponential-based delays.
/// - [`JitterBackoffStrategy`] - An implementation of [`RetryBackoffStrategy`] for jitter-based delays.
#[derive(TypedBuilder)]
#[builder(
    mutators(
        pub fn constant(&mut self, duration: Duration) {
            self.backoff = Box::new(ConstantBackoffStrategy::new(duration));
        }

        pub fn exponential(&mut self, factor: f64) {
            self.backoff = Box::new(ExponentialBackoffStrategy::new(factor));
        }

        pub fn linear(&mut self, factor: Duration) {
            self.backoff = Box::new(
                LinearBackoffStrategy::builder().factor(factor).build()
            );
        }

        pub fn bounded_exponential(&mut self, factor: f64, max: Duration) {
            self.backoff = Box::new(ExponentialBackoffStrategy::new_with(factor, max));
        }

        pub fn bounded_linear(&mut self, factor: Duration, max: Duration) {
            self.backoff = Box::new(
                LinearBackoffStrategy::builder().factor(factor).clamp(max).build()
            );
        }

        pub fn full_jitter(&mut self, backoff: impl RetryBackoffStrategy, factor: f64) {
            self.backoff = Box::new(
                JitterBackoffStrategy::<_>::full(backoff, factor)
            );
        }

        pub fn equal_jitter(&mut self, backoff: impl RetryBackoffStrategy, factor: f64) {
            self.backoff = Box::new(
                JitterBackoffStrategy::<_>::equal(backoff, factor)
            );
        }

        pub fn decorrelated_jitter(&mut self, backoff: impl RetryBackoffStrategy, factor: f64, max: f64) {
            self.backoff = Box::new(
                JitterBackoffStrategy::<_>::decorrelated(backoff, factor, max)
            );
        }

        pub fn backoff(&mut self, backoff: impl RetryBackoffStrategy) {
            self.backoff = Box::new(backoff);
        }
    )
)]
pub struct RetriableTaskFrame<T: TaskFrame> {
    frame: T,

    #[builder(setter(transform = |val: NonZeroU32| val.get()))]
    retries: u32,

    #[builder(via_mutators(init = Box::new(ConstantBackoffStrategy::new(Duration::ZERO))))]
    backoff: Box<dyn RetryBackoffStrategy>,

    #[builder(
        setter(transform = |val: impl RetryErrorFilter<T::Error>|
            Box::new(val) as Box<dyn RetryErrorFilter<T::Error>>
        ),
        default = Box::new(true)
    )]
    filter: Box<dyn RetryErrorFilter<T::Error>>,
}

impl<T: TaskFrame> TaskFrame for RetriableTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let mut error: Result<(), T::Error> = Ok(());

        for retry in 0u32..=self.retries {
            let retry_count = RetryCount(retry);
            ctx.emit::<OnRetryAttemptStart>(&retry_count).await;

            error = self.frame.execute(ctx, args).await;
            let retry_err = RetryError(error.as_ref().map_err(|x| x as &dyn TaskError).err());

            ctx.emit::<OnRetryAttemptEnd>(&(retry_count, retry_err))
                .await;
            let Err(err) = &error else { return Ok(()) };

            if !self.filter.filter(err) {
                break;
            }

            let delay = self.backoff.compute(retry);
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
                continue;
            }

            tokio::task::yield_now().await;
        }

        error
    }
}
