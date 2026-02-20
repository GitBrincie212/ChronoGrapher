use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext, TaskHookEvent};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::clone::Clone;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::time::Duration;
use typed_builder::TypedBuilder;

#[async_trait]
pub trait RetryErrorFilter<T: TaskError>: Send + Sync + 'static {
    async fn execute(&self, error: Option<&T>) -> bool;
}

#[async_trait]
impl<T: TaskError> RetryErrorFilter<T> for () {
    async fn execute(&self, _error: Option<&T>) -> bool {
        true
    }
}

pub trait RetryBackoffStrategy: Debug + Send + Sync + 'static {
    fn compute(&self, retry: u32) -> Duration;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantBackoffStrategy(Duration);

impl ConstantBackoffStrategy {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl RetryBackoffStrategy for ConstantBackoffStrategy {
    fn compute(&self, _retry: u32) -> Duration {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExponentialBackoffStrategy(f64, f64);

impl ExponentialBackoffStrategy {
    pub fn new(factor: f64) -> Self {
        Self(factor, f64::INFINITY)
    }

    pub fn new_with(factor: f64, max_duration: Duration) -> Self {
        Self(factor, max_duration.as_secs_f64())
    }
}

impl RetryBackoffStrategy for ExponentialBackoffStrategy {
    fn compute(&self, retry: u32) -> Duration {
        Duration::from_secs_f64(self.0.powf(retry as f64).min(self.1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearBackoffStrategy(Duration, Duration);

impl LinearBackoffStrategy {
    pub fn new(factor: Duration) -> Self {
        Self(factor, Duration::from_secs_f64(f64::INFINITY))
    }

    pub fn new_with(factor: Duration, max_duration: Duration) -> Self {
        Self(factor, max_duration)
    }
}

impl RetryBackoffStrategy for LinearBackoffStrategy {
    fn compute(&self, retry: u32) -> Duration {
        (self.0 * retry).min(self.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JitterBackoffStrategy<T: RetryBackoffStrategy>(T, f64);

impl<T: RetryBackoffStrategy> JitterBackoffStrategy<T> {
    pub fn new(strat: T, factor: f64) -> Self {
        Self(strat, factor)
    }
}

impl<T: RetryBackoffStrategy> RetryBackoffStrategy for JitterBackoffStrategy<T> {
    fn compute(&self, retry: u32) -> Duration {
        let max_jitter = self.0.compute(retry).mul_f64(self.1);
        Duration::from_secs_f64(fastrand::f64() * max_jitter.as_secs_f64())
    }
}

define_event!(OnRetryAttemptStart, u32);

define_event!(OnRetryAttemptEnd, (u32, Option<&'a dyn TaskError>));

define_event_group!(RetryAttemptEvents, OnRetryAttemptStart, OnRetryAttemptEnd);

#[derive(TypedBuilder)]
#[builder(
    build_method(into = RetriableTaskFrame<T>),
    mutators(
        pub fn constant(&mut self, duration: Duration){
            self.backoff = Box::new(ConstantBackoffStrategy::new(duration));
        }

        pub fn exponential(&mut self, factor: f64){
            self.backoff = Box::new(ExponentialBackoffStrategy::new(factor));
        }

        pub fn linear(&mut self, factor: Duration){
            self.backoff = Box::new(LinearBackoffStrategy::new(factor));
        }

        pub fn bounded_exponential(&mut self, factor: f64, max: Duration){
            self.backoff = Box::new(ExponentialBackoffStrategy::new_with(factor, max));
        }

        pub fn bounded_linear(&mut self, factor: Duration, max: Duration){
            self.backoff = Box::new(LinearBackoffStrategy::new_with(factor, max));
        }

        pub fn backoff(&mut self, backoff: impl RetryBackoffStrategy){
            self.backoff = Box::new(backoff) as Box<dyn RetryBackoffStrategy>;
        }
    )
)]
pub struct RetriableTaskFrameConfig<T: TaskFrame> {
    frame: T,
    retries: NonZeroU32,

    #[builder(via_mutators(init = Box::new(ConstantBackoffStrategy::new(Duration::ZERO))))]
    backoff: Box<dyn RetryBackoffStrategy>,

    #[builder(
        setter(transform = |val: impl RetryErrorFilter<T::Error>|
            Box::new(val) as Box<dyn RetryErrorFilter<T::Error>>
        ),
        default = Box::new(())
    )]
    when: Box<dyn RetryErrorFilter<T::Error>>,
}

impl<T: TaskFrame> From<RetriableTaskFrameConfig<T>> for RetriableTaskFrame<T> {
    fn from(config: RetriableTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            retries: config.retries,
            backoff_strat: config.backoff,
            when: config.when,
        }
    }
}

pub struct RetriableTaskFrame<T: TaskFrame> {
    frame: T,
    retries: NonZeroU32,
    backoff_strat: Box<dyn RetryBackoffStrategy>,
    when: Box<dyn RetryErrorFilter<T::Error>>,
}

impl<T: TaskFrame> RetriableTaskFrame<T> {
    pub fn builder() -> RetriableTaskFrameConfigBuilder<T> {
        RetriableTaskFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for RetriableTaskFrame<T> {
    type Error = T::Error;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let mut error: Result<(), T::Error> = Ok(());
        let subdivided = ctx.subdivided_ctx(&self.frame);
        for retry in 0u32..=self.retries.get() {
            ctx.emit::<OnRetryAttemptStart>(&retry).await;

            error = self.frame.execute(&subdivided).await;
            let erased_err = error.as_ref().err().map(|x| x as &dyn TaskError);

            ctx.emit::<OnRetryAttemptEnd>(&(retry, erased_err)).await;

            if error.is_ok() || !self.when.execute(error.as_ref().err()).await {
                return Ok(());
            }

            if retry == self.retries.get() {
                break;
            }

            let delay = self.backoff_strat.compute(retry);
            tokio::time::sleep(delay).await;
        }

        error
    }
}
