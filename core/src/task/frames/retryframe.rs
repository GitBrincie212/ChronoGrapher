use crate::task::{TaskContext, TaskError, TaskFrame, TaskHookEvent};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::clone::Clone;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use typed_builder::TypedBuilder;

#[async_trait]
pub trait RetryErrorFilter: Send + Sync {
    async fn execute(&self, error: &Option<TaskError>) -> bool;
}

#[async_trait]
impl RetryErrorFilter for () {
    async fn execute(&self, _error: &Option<TaskError>) -> bool {
        true
    }
}

/// [`RetryBackoffStrategy`] is a trait for computing a new delay from when
/// a [`RetriableTaskFrame`] fails and wants to retry. There are multiple
/// implementations to use which can be stacked (tho stacking too many of them doesn't
/// provide flexibility, simplicity is often preferred than more complex retry delay strategies).
///
/// # Required Method(s)
/// When implementing the [`RetryBackoffStrategy`] trait, one has to supply an implementation
/// for the method [`RetryBackoffStrategy::compute`] which is where the logic to compute
/// the delay resides in
///
/// # Trait Implementation(s)
/// There are 3 implementations of [`RetryBackoffStrategy`] trait in the library, those being:
/// - [`ConstantDelayStrategy`] Wraps a duration and gives the same duration
/// - [`ExponentialBackoffStrategy`] For exponential backoff based on a factor
/// - [`JitterBackoffStrategy`] For randomly-based jitter from a backoff strategy
///
/// # See Also
/// - [`RetriableTaskFrame`]
/// - [`ConstantDelayStrategy`]
/// - [`ExponentialBackoffStrategy`]
/// - [`JitterBackoffStrategy`]
#[async_trait]
pub trait RetryBackoffStrategy: Debug + Send + Sync + 'static {
    async fn compute(&self, retry: u32) -> Duration;
}

#[async_trait]
impl<RBS: RetryBackoffStrategy + ?Sized> RetryBackoffStrategy for Arc<RBS> {
    async fn compute(&self, retry: u32) -> Duration {
        self.as_ref().compute(retry).await
    }
}

/// [`ConstantBackoffStrategy`] is an implementation of the [`RetryBackoffStrategy`],
/// essentially wrapping a [`Duration`]
///
/// # Constructor(s)
/// One can simply use [`ConstantBackoffStrategy::new`] to construct a new
/// [`ConstantBackoffStrategy`] instance with a supplied duration
///
/// # Trait Implementation(s)
/// Obviously [`ConstantBackoffStrategy`] implements the trait [`RetryBackoffStrategy`], but
/// also [`Debug`], [`Clone`], [`Copy`], [`PartialEq`] and [`Eq`]
///
/// # See Also
/// - [`RetryBackoffStrategy`]
/// - [`ConstantBackoffStrategy::new`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantBackoffStrategy(Duration);

impl ConstantBackoffStrategy {
    /// Constructs / Creates a new [`ConstantBackoffStrategy`] instance
    ///
    /// # Argument(s)
    /// This method accepts only one argument, that being the ``duration``,
    /// which is a constant delay to apply
    ///
    /// # Returns
    /// The fully constructed [`ConstantBackoffStrategy`] with a constant
    /// delay of ``duration``
    ///
    /// # See Also
    /// - [`ConstantBackoffStrategy`]
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

#[async_trait]
impl RetryBackoffStrategy for ConstantBackoffStrategy {
    async fn compute(&self, _retry: u32) -> Duration {
        self.0
    }
}

/// [`ExponentialBackoffStrategy`] is an implementation of the [`RetryBackoffStrategy`], essentially
/// the more retries happen throughout, the more the duration grows by a specified factor til it reaches
/// a specified maximum threshold in which it will remain constant
///
/// # Constructor(s)
/// There are two constructors to use, if one wants boundless duration, then [`ExponentialBackoffStrategy::new`]
/// is used for convenience, otherwise [`ExponentialBackoffStrategy::new_with`] to specify a maximum
/// threshold
///
/// # Trait Implementation(s)
/// Obviously [`ExponentialBackoffStrategy`] implements the trait [`RetryBackoffStrategy`], but
/// also [`Debug`], [`Clone`], [`Copy`], [`PartialEq`]
///
/// # See Also
/// - [`RetryBackoffStrategy`]
/// - [`ExponentialBackoffStrategy::new`]
/// - [`ExponentialBackoffStrategy::new_with`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExponentialBackoffStrategy(f64, f64);

impl ExponentialBackoffStrategy {
    /// Constructs / Creates a new [`ExponentialBackoffStrategy`] instance
    ///
    /// # Argument(s)
    /// This method accepts one argument, that is the ``factor`` which determines
    /// how much the duration should grow per retry
    ///
    /// # Returns
    /// A fully constructed [`ExponentialBackoffStrategy`] with no maximum threshold
    /// and a growth factor as ``factor``
    ///
    /// # See Also
    /// - [`ExponentialBackoffStrategy`]
    pub fn new(factor: f64) -> Self {
        Self(factor, f64::INFINITY)
    }

    /// Constructs / Creates a new [`ExponentialBackoffStrategy`] instance
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being the ``factor`` which determines
    /// how much the duration should grow per retry and the ``max_duration`` which is a threshold
    /// / ceiling the duration cannot surpass
    ///
    /// # Returns
    /// A fully constructed [`ExponentialBackoffStrategy`] with maximum threshold of ``max_duration``
    /// and a growth factor as ``factor``
    ///
    /// # See Also
    /// - [`ExponentialBackoffStrategy`]
    pub fn new_with(factor: f64, max_duration: f64) -> Self {
        Self(factor, max_duration)
    }
}

#[async_trait]
impl RetryBackoffStrategy for ExponentialBackoffStrategy {
    async fn compute(&self, retry: u32) -> Duration {
        Duration::from_secs_f64(self.0.powf(retry as f64).min(self.1))
    }
}

/// [`JitterBackoffStrategy`] is an implementation of [`RetryBackoffStrategy`], acting as a wrapper
/// around a backoff strategy, essentially it distorts the results by a specified randomness factor
///
/// # Constructor(s)
/// The only way to construct a [`JitterBackoffStrategy`] is via [`JitterBackoffStrategy::new`]
/// with a provided randomness / jiter factor
///
/// # Trait Implementation(s)
/// Obviously [`JitterBackoffStrategy`] implements the trait [`RetryBackoffStrategy`], but
/// also [`Debug`], [`Clone`] and [`Copy`]
///
/// # See Also
/// - [`JitterBackoffStrategy::new`]
/// - [`RetryBackoffStrategy`]
#[derive(Debug, Clone, Copy)]
pub struct JitterBackoffStrategy<T: RetryBackoffStrategy>(T, f64);

impl<T: RetryBackoffStrategy> JitterBackoffStrategy<T> {
    /// Creates / Constructs a new [`JitterBackoffStrategy`]
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being a wrapped ``strat``
    /// of type [`RetryBackoffStrategy`] and a jittered ``factor`` for
    /// how much noise to add to the computed result
    ///
    /// # Returns
    /// A fully constructed [`JitterBackoffStrategy`], with a wrapped [`RetryBackoffStrategy`]
    /// as ``strat`` and a noise / jittered factor being ``factor``
    ///
    /// # See Also
    /// - [`JitterBackoffStrategy`]
    pub fn new(strat: T, factor: f64) -> Self {
        Self(strat, factor)
    }
}

#[async_trait]
impl<T: RetryBackoffStrategy> RetryBackoffStrategy for JitterBackoffStrategy<T> {
    async fn compute(&self, retry: u32) -> Duration {
        let max_jitter = self.0.compute(retry).await.mul_f64(self.1);
        Duration::from_secs_f64(fastrand::f64() * max_jitter.as_secs_f64())
    }
}

define_event!(
    /// [`OnRetryAttemptStart`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnRetryAttemptStart`]
    /// is ``u32`` which is the number of retries that have occurred
    ///
    /// # Constructor(s)
    /// When constructing a [`OnRetryAttemptStart`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnRetryAttemptStart::default`] or via simply pasting
    /// the struct name ([`OnRetryAttemptStart`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnRetryAttemptStart`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnRetryAttemptStart`] is triggered when the [`RetriableTaskFrame`] is attempting
    /// to retry executing the wrapped [`TaskFrame`] in an effort to see if it succeeds or fails
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnRetryAttemptStart`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`RetriableTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnRetryAttemptStart, u32
);

define_event!(
    /// [`OnRetryAttemptEnd`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnRetryAttemptEnd`]
    /// is ``(u32, Option<TaskError>)``, the first value describes the number of retries
    /// that have occurred and the second value is a potential error returned from the TaskFrame
    /// where ``Some(...)`` means failure and ``None`` means success
    ///
    /// # Constructor(s)
    /// When constructing a [`OnRetryAttemptEnd`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnRetryAttemptEnd::default`] or via simply pasting
    /// the struct name ([`OnRetryAttemptEnd`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnRetryAttemptEnd`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnRetryAttemptEnd`] is triggered when the [`RetriableTaskFrame`] has attempted
    /// to retry executing the wrapped [`TaskFrame`] and the results came in (i.e. A potential
    /// error from the execution)
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnRetryAttemptEnd`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`RetriableTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnRetryAttemptEnd, (u32, Option<TaskError>)
);

define_event_group!(
    /// [`RetryAttemptEvents`] is a marker trait, more specifically a [`TaskHookEvent`] group of
    /// [`TaskHookEvent`] (a system used closely with [`TaskHook`]). It contains no common payload type
    ///
    /// # Supertrait(s)
    /// Since it is a [`TaskHookEvent`] group, it requires every descended to implement the [`TaskHookEvent`],
    /// because no common payload type is present, any payload type is accepted
    ///
    /// # Trait Implementation(s)
    /// Currently, two [`TaskHookEvent`] implement the [`RetryAttemptEvents`] marker trait
    /// (event group). Those being [`OnDelayStart`] and [`OnDelayEnd`]
    ///
    /// # Object Safety
    /// [`RetryAttemptEvents`] is **NOT** object safe, due to the fact it implements the
    /// [`TaskHookEvent`] which itself is not object safe
    ///
    /// # See Also
    /// - [`OnDelayStart`]
    /// - [`OnDelayEnd`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    RetryAttemptEvents,
    OnRetryAttemptStart, OnRetryAttemptEnd
);

#[derive(TypedBuilder)]
#[builder(build_method(into = RetriableTaskFrame<T1, T2, T3>))]
pub struct RetriableTaskFrameConfig<
    T1: TaskFrame,
    T2: RetryBackoffStrategy,
    T3: RetryErrorFilter
> {
    frame: T1,
    retries: NonZeroU32,
    backoff_strat: T2,
    when: T3
}

impl<T1, T2, T3> From<RetriableTaskFrameConfig<T1, T2, T3>> for RetriableTaskFrame<T1, T2, T3>
where
    T1: TaskFrame,
    T2: RetryBackoffStrategy,
    T3: RetryErrorFilter
{
    fn from(config: RetriableTaskFrameConfig<T1, T2, T3>) -> Self {
        Self {
            frame: Arc::new(config.frame),
            retries: config.retries,
            backoff_strat: config.backoff_strat,
            when: config.when,
        }
    }
}

/// Represents a **retriable task frame** which wraps a [`TaskFrame`]. This task frame type acts as a
/// **wrapper node** within the task frame hierarchy, providing a retry mechanism for execution.
///
/// # Behavior
/// - Executes the **wrapped task frame**.
/// - If the task frame fails, it re-executes it again after a specified delay (or instantaneous).
/// - Repeat the process for a specified number of retries til the task frame succeeds
///
/// # Constructor(s)
/// When constructing a [`RetriableTaskFrame`], one can use 3 constructors at their disposal:
/// - [`RetriableTaskFrame::new`] Creates a [`RetriableTaskFrame`] with a
///   constant delay per retry and a specified number of retries
/// - [`RetriableTaskFrame::new_instant`] Creates a [`RetriableTaskFrame`] with a
///   no delay per retry and a specified number of retries
/// - [`RetriableTaskFrame::new_with`] Creates a [`RetriableTaskFrame`] with a
///   custom [`RetryBackoffStrategy`] and a specified number of retries
///
/// # Events
/// [`RetriableTaskFrame`] provides 2 events, namely ``on_retry_start`` which executes when a retry
/// happens, it hands out the wrapped task frame instance. As well as the ``on_retry_end`` which
/// executes when a retry is finished, it hands out the wrapped task frame instance and an option
/// error for a potential error it may have gotten from this retry
///
/// # Trait Implementation(s)
/// It is obvious that the [`RetriableTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however, it also implements
/// [`PersistenceObject`], [`Serialize`] and [`Deserialize`]. ONLY if the underlying
/// [`TaskFrame`] and [`RetryBackoffStrategy`] are persistable
///
/// # Example
/// ```ignore
/// use std::num::NonZeroU32;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::retryframe::RetriableTaskFrame;
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::Task;
///
/// let exec_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Trying primary task...");
///         Err::<(), ()>(())
///     }
/// );
///
/// let retriable_frame = RetriableTaskFrame::new_instant(
///     exec_frame,
///     NonZeroU32::new(3).unwrap(), // We know it isn't zero, so safe to unwrap
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(2.5), retriable_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
///
/// # See Also
/// - [`TaskFrame`]
/// - [`RetryBackoffStrategy`]
pub struct RetriableTaskFrame<
    T1: TaskFrame,
    T2: RetryBackoffStrategy = ConstantBackoffStrategy,
    T3: RetryErrorFilter = ()
> {
    frame: Arc<T1>,
    retries: NonZeroU32,
    backoff_strat: T2,
    when: T3
}

type IncompleteFilterlessInstantBuilder<T> = RetriableTaskFrameConfigBuilder<
    T, ConstantBackoffStrategy, (),
    ((), (), (ConstantBackoffStrategy,), ((),))
>;

type IncompleteInstantBuilder<T1, T2: RetryErrorFilter> = RetriableTaskFrameConfigBuilder<
    T1, ConstantBackoffStrategy, T2,
    ((), (), (ConstantBackoffStrategy,), ())
>;

type IncompleteFilterlessBuilder<T1, T2: RetryBackoffStrategy> = RetriableTaskFrameConfigBuilder<
    T1, T2, (),
    ((), (), (), ((),))
>;

impl<T1: TaskFrame, T2: RetryBackoffStrategy> RetriableTaskFrame<T1, T2> {
    pub fn filterless_builder() -> IncompleteFilterlessBuilder<T1, T2>
    {
        RetriableTaskFrameConfig::builder()
            .when(())
    }
}


impl<T1: TaskFrame, T2: RetryErrorFilter> RetriableTaskFrame<T1, ConstantBackoffStrategy, T2> {
    pub fn instant_builder() -> IncompleteInstantBuilder<T1, T2>
    {
        RetriableTaskFrameConfig::builder()
            .backoff_strat(ConstantBackoffStrategy::new(Duration::ZERO))
    }
}


impl<T: TaskFrame> RetriableTaskFrame<T> {
    pub fn instant_filterless_builder() -> IncompleteFilterlessInstantBuilder<T>
    {
        RetriableTaskFrameConfig::builder()
            .backoff_strat(ConstantBackoffStrategy::new(Duration::ZERO))
            .when(())
    }
}

impl<T1: TaskFrame, T2: RetryBackoffStrategy, T3: RetryErrorFilter> RetriableTaskFrame<T1, T2, T3> {
    pub fn builder() -> RetriableTaskFrameConfigBuilder<T1, T2, T3> {
        RetriableTaskFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame, T2: RetryBackoffStrategy> TaskFrame for RetriableTaskFrame<T, T2> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        let mut error: Option<TaskError> = None;
        let subdivided = ctx.subdivided_ctx(self.frame.clone());
        for retry in 0u32..=self.retries.get() {
            ctx.emit::<OnRetryAttemptStart>(&retry).await;

            error = self.frame.execute(&subdivided).await.err();

            ctx.emit::<OnRetryAttemptEnd>(&(retry, error.clone())).await;

            if error.is_none() || !self.when.execute(&error).await {
                return Ok(());
            }

            if retry == self.retries.get() {
                break;
            }

            let delay = self.backoff_strat.compute(retry).await;
            tokio::time::sleep(delay).await;
        }

        error.map_or(Ok(()), Err)
    }
}
