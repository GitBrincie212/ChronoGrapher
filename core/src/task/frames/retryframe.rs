use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use async_trait::async_trait;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// [`RetryBackoffStrategy`] is a trait for computing a new delay from when
/// a [`RetriableTaskFrame`] fails and wants to retry. There are multiple
/// implementations to use which can be stacked (tho stacking too many of them doesn't
/// provide flexibility, simplicity is often preferred than more complex retry delay strategies).
///
/// Some example implementations are:
/// - [`ConstantDelayStrategy`] Wraps a duration and gives the same duration
/// - [`ExponentialBackoffStrategy`] For exponential backoff based on a factor
/// - [`JitterBackoffStrategy`] For randomly-based jitter from a backoff strategy
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
#[derive(Debug)]
pub struct ConstantBackoffStrategy(Duration);

impl ConstantBackoffStrategy {
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
#[derive(Debug)]
pub struct ExponentialBackoffStrategy(f64, f64);

impl ExponentialBackoffStrategy {
    pub fn new(factor: f64) -> Self {
        Self(factor, f64::INFINITY)
    }

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
#[derive(Debug)]
pub struct JitterBackoffStrategy<T: RetryBackoffStrategy>(T, f64);

impl<T: RetryBackoffStrategy> JitterBackoffStrategy<T> {
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

/// Represents a **retriable task frame** which wraps a task frame. This task frame type acts as a
/// **wrapper node** within the task frame hierarchy, providing a retry mechanism for execution.
///
/// # Behavior
/// - Executes the **wrapped task frame**.
/// - If the task frame fails, it re-executes it again after a specified delay (or instantaneous).
/// - Repeat the process for a specified number of retries til the task frame succeeds
///
/// # Events
/// [`RetriableTaskFrame`] provides 2 events, namely ``on_retry_start`` which executes when a retry
/// happens, it hands out the wrapped task frame instance. As well as the ``on_retry_end`` which
/// executes when a retry is finished, it hands out the wrapped task frame instance and an option
/// error for a potential error it may have gotten from this retry
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
///     |_metadata| async {
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
pub struct RetriableTaskFrame<T: 'static, T2: RetryBackoffStrategy = ConstantBackoffStrategy> {
    task: Arc<T>,
    retries: NonZeroU32,
    backoff_strat: T2,
    pub on_retry_start: ArcTaskEvent<Arc<T>>,
    pub on_retry_end: ArcTaskEvent<(Arc<T>, Option<TaskError>)>,
}

impl<T: TaskFrame + 'static> RetriableTaskFrame<T> {
    /// Creates a retriable task that has a specified delay per retry
    pub fn new(task: T, retries: NonZeroU32, delay: Duration) -> Self {
        Self {
            task: Arc::new(task),
            retries,
            backoff_strat: ConstantBackoffStrategy::new(delay),
            on_retry_end: TaskEvent::new(),
            on_retry_start: TaskEvent::new(),
        }
    }

    /// Creates a retriable task that has no delay per retry
    pub fn new_instant(task: T, retries: NonZeroU32) -> Self {
        RetriableTaskFrame::<T, ConstantBackoffStrategy>::new(task, retries, Duration::ZERO)
    }
}

impl<T: TaskFrame + 'static, T2: RetryBackoffStrategy> RetriableTaskFrame<T, T2> {
    /// Creates a retriable task that has a specific backoff strategy per retry
    pub fn new_with(task: T, retries: NonZeroU32, backoff_strat: T2) -> Self {
        Self {
            task: Arc::new(task),
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
        for retry in 0u32..self.retries.get() {
            if retry != 0 {
                ctx.emitter
                    .emit(
                        ctx.metadata.clone(),
                        self.on_retry_start.clone(),
                        self.task.clone(),
                    )
                    .await;
            }
            let result = self.task.execute(ctx.clone()).await;
            match result {
                Ok(_) => {
                    ctx.emitter
                        .emit(
                            ctx.metadata.clone(),
                            self.on_retry_end.clone(),
                            (self.task.clone(), None),
                        )
                        .await;
                    return Ok(());
                }
                Err(err) => {
                    error = Some(err.clone());
                    ctx.emitter
                        .emit(
                            ctx.metadata.clone(),
                            self.on_retry_end.clone(),
                            (self.task.clone(), error.clone()),
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
