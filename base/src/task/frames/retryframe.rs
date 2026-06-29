use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext, TaskHookEvent};
use crate::utils::macros::{define_event, define_event_group};
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

#[async_trait]
impl<F, Fut, T> RetryErrorFilter<T> for F
where
    T: TaskError,
    Fut: Future<Output = bool> + Send + Sync + 'static,
    F: Fn(Option<&T>) -> Fut + Send + Sync + 'static,
{
    async fn execute(&self, error: Option<&T>) -> bool {
        self(error).await
    }
}

pub trait RetryBackoffStrategy: Send + Sync + 'static {
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

#[derive(TypedBuilder)]
#[builder(build_method(into = LinearBackoffStrategy))]
pub struct LinearBackoffStrategyConfig {
    factor: Duration,

    #[builder(default = Duration::ZERO)]
    start: Duration,

    #[builder(default, setter(strip_option))]
    clamp: Option<Duration>,
}

impl Into<LinearBackoffStrategy> for LinearBackoffStrategyConfig {
    fn into(self) -> LinearBackoffStrategy {
        let start = self.start.as_secs_f64();
        let factor = self.factor.as_secs_f64();
        let clamp = self.clamp.map(|x| x.as_secs_f64()).unwrap_or(f64::INFINITY);

        LinearBackoffStrategy {
            start,
            factor,
            clamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy)]
enum JitterType {
    FullJitter,
    EqualJitter,
    DecorrelatedJitter(f64),
}

#[derive(Clone, Copy)]
pub struct JitterBackoffStrategy<T: RetryBackoffStrategy> {
    backoff: T,
    factor: f64,
    jitter_type: JitterType,
}

impl<T: RetryBackoffStrategy> JitterBackoffStrategy<T> {
    pub fn new_full(strat: T, factor: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::FullJitter,
        }
    }

    pub fn new_equal(strat: T, factor: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::EqualJitter,
        }
    }

    pub fn new_decorrelated(strat: T, factor: f64, max: f64) -> Self {
        Self {
            backoff: strat,
            factor,
            jitter_type: JitterType::DecorrelatedJitter(max),
        }
    }
}

impl<T: RetryBackoffStrategy> RetryBackoffStrategy for JitterBackoffStrategy<T> {
    fn compute(&self, retry: u32) -> Duration {
        let base = self.backoff.compute(retry).mul_f64(self.factor);

        let base_secs = base.as_secs_f64();

        let secs = match self.jitter_type {
            JitterType::FullJitter => fastrand::f64() * base_secs,

            JitterType::EqualJitter => {
                let half = base_secs / 2.0;
                half + (fastrand::f64() * half)
            }

            JitterType::DecorrelatedJitter(max) => {
                // TODO: This is an approximation, might get fixed in the future
                let upper = (base_secs * 3.0).min(max);

                fastrand::f64() * upper
            }
        };

        Duration::from_secs_f64(secs)
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
            self.backoff = Box::new(
                LinearBackoffStrategy::builder().factor(factor).build()
            );
        }

        pub fn bounded_exponential(&mut self, factor: f64, max: Duration){
            self.backoff = Box::new(ExponentialBackoffStrategy::new_with(factor, max));
        }

        pub fn bounded_linear(&mut self, factor: Duration, max: Duration){
            self.backoff = Box::new(
                LinearBackoffStrategy::builder().factor(factor).clamp(max).build()
            );
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

impl<T: TaskFrame> TaskFrame for RetriableTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let mut error: Result<(), T::Error> = Ok(());

        for retry in 0u32..=self.retries.get() {
            ctx.emit::<OnRetryAttemptStart>(&retry).await;

            error = self.frame.execute(&ctx, &args).await;
            let erased_err = error.as_ref().map_err(|x| x as &dyn TaskError).err();

            ctx.emit::<OnRetryAttemptEnd>(&(retry, erased_err)).await;

            if error.is_ok() || !self.when.execute(error.as_ref().err()).await {
                return Ok(());
            }

            if retry == self.retries.get() {
                break;
            }

            let delay = self.backoff_strat.compute(retry);
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            } else {
                tokio::task::yield_now().await;
            }
        }

        error
    }
}
