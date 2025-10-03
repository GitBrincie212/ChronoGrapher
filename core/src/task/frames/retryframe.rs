use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use crate::{acquire_mut_ir_map, deserialization_err, deserialize_field, to_json};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::clone::Clone;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
        Duration::from_secs_f64(rand::random::<f64>() * max_jitter.as_secs_f64())
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
/// is a part of the default provided implementations, however there are many others
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
pub struct RetriableTaskFrame<T: 'static, T2: RetryBackoffStrategy = ConstantBackoffStrategy> {
    frame: Arc<T>,
    retries: NonZeroU32,
    backoff_strat: T2,

    /// Event fired when a retry occurs for the wrapped [`TaskFrame`],
    /// hosting the wrapped [`TaskFrame`] instance
    pub on_retry_start: ArcTaskEvent<Arc<T>>,

    /// Event fired when a retry ends for the wrapped [`TaskFrame`],
    /// hosting the wrapped [`TaskFrame`] instance as well as an option error
    /// it may have thrown
    pub on_retry_end: ArcTaskEvent<(Arc<T>, Option<TaskError>)>,
}

impl<T: TaskFrame + 'static> RetriableTaskFrame<T> {
    /// Creates / Constructs a [`RetriableTaskFrame`] that has a specified delay per retry
    ///
    /// # Argument(s)
    /// This method accepts 3 arguments, the first being the [`TaskFrame`] as ``frame``, the second
    /// being the number of retries as ``retries`` and the third being a constant delay as ``delay``
    ///
    /// # Returns
    /// The constructed [`RetriableTaskFrame`] instance that wraps a [`TaskFrame`] as ``frame`` which
    /// will be retried ``retries`` times until it succeeds with each retry having a delay of ``delay``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetriableTaskFrame`]
    pub fn new(frame: T, retries: NonZeroU32, delay: Duration) -> Self {
        Self {
            frame: Arc::new(frame),
            retries,
            backoff_strat: ConstantBackoffStrategy::new(delay),
            on_retry_end: TaskEvent::new(),
            on_retry_start: TaskEvent::new(),
        }
    }

    /// Creates / Constructs a [`RetriableTaskFrame`] that has no delay per retry
    ///
    /// # Argument(s)
    /// This method accepts 2 arguments, the first being the [`TaskFrame`] as ``frame`` and
    /// the second being the number of retries as ``retries``
    ///
    /// # Returns
    /// The constructed [`RetriableTaskFrame`] instance that wraps a [`TaskFrame`] as ``frame`` which
    /// will be retried ``retries`` times until it succeeds with no delay in between
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetriableTaskFrame`]
    pub fn new_instant(task: T, retries: NonZeroU32) -> Self {
        RetriableTaskFrame::<T, ConstantBackoffStrategy>::new(task, retries, Duration::ZERO)
    }
}

impl<T: TaskFrame + 'static, T2: RetryBackoffStrategy> RetriableTaskFrame<T, T2> {
    /// Creates / Constructs a [`RetriableTaskFrame`] that has a custom backoff strategy per retry
    ///
    /// # Argument(s)
    /// This method accepts 3 arguments, the first being the [`TaskFrame`] as ``frame``, the second
    /// being the number of retries as ``retries`` and the third being a custom [`RetryBackoffStrategy`]
    /// as ``backoff_strat``
    ///
    /// # Returns
    /// The constructed [`RetriableTaskFrame`] instance that wraps a [`TaskFrame`] as ``frame`` which
    /// will be retried ``retries`` times until it succeeds with each retry's delay being computed
    /// in a [`RetryBackoffStrategy`] via ``backoff_strat``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`RetryBackoffStrategy`]
    /// - [`RetriableTaskFrame`]
    pub fn new_with(task: T, retries: NonZeroU32, backoff_strat: T2) -> Self {
        Self {
            frame: Arc::new(task),
            retries,
            backoff_strat,
            on_retry_end: TaskEvent::new(),
            on_retry_start: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl<T: TaskFrame + 'static, T2: RetryBackoffStrategy> TaskFrame for RetriableTaskFrame<T, T2> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let mut error: Option<TaskError> = None;
        let restricted_context = ctx.as_restricted();
        for retry in 0u32..self.retries.get() {
            if retry != 0 {
                ctx.emitter
                    .emit(
                        restricted_context.clone(),
                        self.on_retry_start.clone(),
                        self.frame.clone(),
                    )
                    .await;
            }
            let result = self.frame.execute(ctx.clone()).await;
            match result {
                Ok(_) => {
                    ctx.emitter
                        .emit(
                            restricted_context.clone(),
                            self.on_retry_end.clone(),
                            (self.frame.clone(), None),
                        )
                        .await;
                    return Ok(());
                }
                Err(err) => {
                    error = Some(err.clone());
                    ctx.emitter
                        .emit(
                            restricted_context.clone(),
                            self.on_retry_end.clone(),
                            (self.frame.clone(), error.clone()),
                        )
                        .await;
                }
            }
            let delay = self.backoff_strat.compute(retry).await;
            tokio::time::sleep(delay).await;
        }
        Err(error.unwrap())
    }
}

#[async_trait]
impl<T1, T2> PersistentObject for RetriableTaskFrame<T1, T2>
where
    T1: TaskFrame + 'static + PersistentObject,
    T2: RetryBackoffStrategy + PersistentObject,
{
    fn persistence_id() -> &'static str {
        "RetriableTaskFrame$chronographer_core"
    }

    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        let serialized_backoff_strat = to_json!(self.backoff_strat.store().await?);

        let serialized_frame = to_json!(self.frame.store().await?);

        let payload = json!({
            "retry_backoff_strategy": serialized_backoff_strat,
            "wrapped_task_frame": serialized_frame,
            "retries": self.retries.get()
        });

        Ok(SerializedComponent::new::<Self>(payload))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = acquire_mut_ir_map!(DelayTaskFrame, component);

        deserialize_field!(
            repr,
            serialized_retries,
            "retries",
            RetriableTaskFrame,
            "Cannot deserialize the number of retries"
        );

        deserialize_field!(
            repr,
            serialized_frame,
            "wrapped_task_frame",
            RetriableTaskFrame,
            "Cannot deserialize the wrapped task frame"
        );

        deserialize_field!(
            repr,
            serialized_backoff_strat,
            "retry_backoff_strategy",
            RetriableTaskFrame,
            "Cannot deserialize the retry backoff strategy"
        );

        let retries = serialized_retries.as_u64().ok_or_else(|| {
            let err = ChronographerErrors::DeserializationFailed(
                "RetriableTaskFrame".to_string(),
                "Cannot deserialize the number of retries".to_string(),
                repr.clone(),
            );

            Arc::new(err) as Arc<dyn Debug + Send + Sync>
        })? as u32;

        let nonzero_retries = NonZeroU32::new(retries).ok_or_else(|| {
            let err = ChronographerErrors::DeserializationFailed(
                "RetriableTaskFrame".to_string(),
                "The deserialized number of retries is zero".to_string(),
                repr.clone(),
            );

            Arc::new(err) as Arc<dyn Debug + Send + Sync>
        })?;

        let frame = T1::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_frame.clone())
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await?;

        let retry_backoff_strat = T2::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_backoff_strat.clone())
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await?;

        Ok(RetriableTaskFrame::new_with(
            frame,
            nonzero_retries,
            retry_backoff_strat,
        ))
    }
}
