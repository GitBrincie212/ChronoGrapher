use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use typed_builder::TypedBuilder;
use crate::errors::ChronographerErrors;
use crate::prelude::TaskContext;
use crate::task::{DynArcError, TaskFrame};

#[async_trait]
pub trait ThresholdLogic: Send + Sync {
    fn counts(&self, res: Option<DynArcError>, ctx: &TaskContext) -> bool;
}

pub struct ThresholdErrorsCountLogic;

#[async_trait]
impl ThresholdLogic for ThresholdErrorsCountLogic {
    fn counts(&self, res: Option<DynArcError>, _ctx: &TaskContext) -> bool {
        res.is_some()
    }
}

pub struct ThresholdSuccessesCountLogic;

#[async_trait]
impl ThresholdLogic for ThresholdSuccessesCountLogic {
    fn counts(&self, res: Option<DynArcError>, _ctx: &TaskContext) -> bool {
        res.is_none()
    }
}

pub struct ThresholdIdentityCountLogic;

#[async_trait]
impl ThresholdLogic for ThresholdIdentityCountLogic {
    fn counts(&self, _res: Option<DynArcError>, _ctx: &TaskContext) -> bool {
        true
    }
}

#[async_trait]
pub trait ThresholdReachBehaviour: Send + Sync {
    fn results(&self, ctx: &TaskContext) -> Result<(), DynArcError>;
}

pub struct ThresholdSuccessReachBehaviour;

#[async_trait]
impl ThresholdReachBehaviour for ThresholdSuccessReachBehaviour {
    fn results(&self, _: &TaskContext) -> Result<(), DynArcError> {
        Ok(())
    }
}

pub struct ThresholdErrorReachBehaviour;

#[async_trait]
impl ThresholdReachBehaviour for ThresholdErrorReachBehaviour {
    fn results(&self, _: &TaskContext) -> Result<(), DynArcError> {
        Err(Arc::new(ChronographerErrors::ThresholdReachError))
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = ThresholdTaskFrame<T>))]
pub struct ThresholdFrameConfig<T: TaskFrame> {
    frame: T,

    #[builder(default = Arc::new(ThresholdIdentityCountLogic))]
    threshold_logic: Arc<dyn ThresholdLogic>,

    #[builder(default = Arc::new(ThresholdSuccessReachBehaviour))]
    threshold_reach_behaviour: Arc<dyn ThresholdReachBehaviour>,
    threshold: NonZeroUsize,
}

impl<T: TaskFrame> From<ThresholdFrameConfig<T>> for ThresholdTaskFrame<T> {
    fn from(config: ThresholdFrameConfig<T>) -> Self {
        Self {
            frame: Arc::new(config.frame),
            threshold_logic: config.threshold_logic,
            threshold_reach_behaviour: config.threshold_reach_behaviour,
            threshold: config.threshold,
            count: AtomicUsize::new(0)
        }
    }
}

pub struct ThresholdTaskFrame<T: TaskFrame> {
    frame: Arc<T>,
    threshold_logic: Arc<dyn ThresholdLogic>,
    threshold_reach_behaviour: Arc<dyn ThresholdReachBehaviour>,
    threshold: NonZeroUsize,
    count: AtomicUsize
}

impl<T: TaskFrame> ThresholdTaskFrame<T> {
    pub fn builder() -> ThresholdFrameConfigBuilder<T> {
        ThresholdFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for ThresholdTaskFrame<T> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), DynArcError> {
        let mut total = self.count.load(Ordering::Relaxed);
        if total == self.threshold.get() {
            return self.threshold_reach_behaviour.results(ctx);
        }
        let res = ctx.subdivide(self.frame.clone()).await;
        if self.threshold_logic.counts(res.clone().err(), ctx) {
            self.count.fetch_add(1, Ordering::SeqCst);
            total += 1;
        }
        if total == self.threshold.get() && ctx.depth == 0 {
            // TODO: Use the handle from the scheduler to cancel the entire workflow
        }
        res
    }
}