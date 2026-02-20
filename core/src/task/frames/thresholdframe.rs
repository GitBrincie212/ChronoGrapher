use crate::errors::TaskError;
use crate::task::{RestrictTaskFrameContext, TaskFrame, TaskFrameContext};
use async_trait::async_trait;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use typed_builder::TypedBuilder;

#[async_trait]
pub trait ThresholdLogic<T: TaskError>: Send + Sync {
    async fn counts(&self, res: Option<&T>, ctx: &RestrictTaskFrameContext) -> bool;
}

macro_rules! impl_error_count_logic {
    ($name: ident, $code: expr) => {
        pub struct $name;

        #[async_trait]
        impl<T: TaskError> ThresholdLogic<T> for $name {
            async fn counts(&self, res: Option<&T>, ctx: &RestrictTaskFrameContext) -> bool {
                ($code)(res, ctx)
            }
        }
    };
}

impl_error_count_logic!(ThresholdErrorsCountLogic, |res: Option<&T>, _| res
    .is_some());
impl_error_count_logic!(ThresholdSuccessesCountLogic, |res: Option<&T>, _| res
    .is_none());
impl_error_count_logic!(ThresholdIdentityCountLogic, |_: Option<&T>, _| true);

#[async_trait]
pub trait ThresholdReachBehaviour<T: TaskError>: Send + Sync {
    async fn results(&self, ctx: &RestrictTaskFrameContext) -> Result<(), T>;
}

macro_rules! impl_threshold_reach_logic {
    ($name: ident, $code: expr) => {
        pub struct $name;

        #[async_trait]
        impl<T: TaskError> ThresholdReachBehaviour<T> for $name {
            async fn results(&self, ctx: &RestrictTaskFrameContext) -> Result<(), T> {
                ($code)(ctx)
            }
        }
    };
}

impl_threshold_reach_logic!(ThresholdSuccessReachBehaviour, |_| Ok(()));

#[derive(TypedBuilder)]
#[builder(build_method(into = ThresholdTaskFrame<T>))]
pub struct ThresholdFrameConfig<T: TaskFrame> {
    frame: T,

    #[builder(default = Box::new(ThresholdIdentityCountLogic))]
    threshold_logic: Box<dyn ThresholdLogic<T::Error>>,

    #[builder(default = Box::new(ThresholdSuccessReachBehaviour))]
    threshold_reach_behaviour: Box<dyn ThresholdReachBehaviour<T::Error>>,
    threshold: NonZeroUsize,
}

impl<T: TaskFrame> From<ThresholdFrameConfig<T>> for ThresholdTaskFrame<T> {
    fn from(config: ThresholdFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            threshold_logic: config.threshold_logic,
            threshold_reach_behaviour: config.threshold_reach_behaviour,
            threshold: config.threshold,
            count: AtomicUsize::new(0),
        }
    }
}

pub struct ThresholdTaskFrame<T: TaskFrame> {
    frame: T,
    threshold_logic: Box<dyn ThresholdLogic<T::Error>>,
    threshold_reach_behaviour: Box<dyn ThresholdReachBehaviour<T::Error>>,
    threshold: NonZeroUsize,
    count: AtomicUsize,
}

impl<T: TaskFrame> ThresholdTaskFrame<T> {
    pub fn builder() -> ThresholdFrameConfigBuilder<T> {
        ThresholdFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for ThresholdTaskFrame<T> {
    type Error = T::Error;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let mut total = self.count.load(Ordering::Relaxed);
        if total == self.threshold.get() {
            return self
                .threshold_reach_behaviour
                .results(&ctx.0)
                .await
        }

        let res = ctx.subdivide(&self.frame).await;
        if self
            .threshold_logic
            .counts(res.as_ref().err(), &ctx.0)
            .await
        {
            self.count.fetch_add(1, Ordering::SeqCst);
            total += 1;
        }

        if total == self.threshold.get() && ctx.depth == 0 {
            // TODO: Use the handle from the scheduler to cancel the entire workflow
        }

        res
    }
}
