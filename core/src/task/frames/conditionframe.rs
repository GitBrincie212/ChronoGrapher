use crate::errors::ConditionalTaskFrameError;
#[allow(unused_imports)]
use crate::task::FallbackTaskFrame;
use crate::task::TaskFrame;
use crate::task::noopframe::NoOperationTaskFrame;
use crate::task::{RestrictTaskFrameContext, TaskFrameContext, TaskHookEvent};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[async_trait]
pub trait ConditionalFramePredicate: Send + Sync {
    async fn execute(&self, ctx: &RestrictTaskFrameContext) -> bool;
}

#[async_trait]
impl<F, Fut> ConditionalFramePredicate for F
where
    F: Fn(&RestrictTaskFrameContext) -> Fut + Send + Sync,
    Fut: Future<Output = bool> + Send,
{
    async fn execute(&self, ctx: &RestrictTaskFrameContext) -> bool {
        self(ctx).await
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = ConditionalFrame<T, T2>))]
pub struct ConditionalFrameConfig<T: TaskFrame, T2: TaskFrame> {
    fallback: T2,

    frame: T,

    #[builder(setter(transform = |s: impl ConditionalFramePredicate + 'static| {
        Arc::new(s) as Arc<dyn ConditionalFramePredicate>
    }))]
    predicate: Arc<dyn ConditionalFramePredicate>,

    #[builder(default = false)]
    error_on_false: bool,
}

define_event!(OnTruthyValueEvent, ());

define_event!(OnFalseyValueEvent, ());

define_event_group!(
    ConditionalPredicateEvents,
    () | OnTruthyValueEvent,
    OnFalseyValueEvent
);

impl<T: TaskFrame, T2: TaskFrame> From<ConditionalFrameConfig<T, T2>> for ConditionalFrame<T, T2> {
    fn from(config: ConditionalFrameConfig<T, T2>) -> Self {
        ConditionalFrame {
            frame: config.frame,
            fallback: config.fallback,
            predicate: config.predicate,
            error_on_false: config.error_on_false,
        }
    }
}

pub struct ConditionalFrame<T, T2> {
    frame: T,
    fallback: T2,
    predicate: Arc<dyn ConditionalFramePredicate>,
    error_on_false: bool,
}

#[allow(type_alias_bounds)]
pub type NonFallbackCFCBuilder<T: TaskFrame> = ConditionalFrameConfigBuilder<
    T,
    NoOperationTaskFrame<T::Error>,
    ((NoOperationTaskFrame<T::Error>,), (), (), ()),
>;

impl<T: TaskFrame> ConditionalFrame<T, NoOperationTaskFrame<T::Error>> {
    pub fn builder() -> NonFallbackCFCBuilder<T> {
        ConditionalFrameConfig::builder().fallback(NoOperationTaskFrame::default())
    }
}

impl<T: TaskFrame, T2: TaskFrame> ConditionalFrame<T, T2> {
    pub fn fallback_builder() -> ConditionalFrameConfigBuilder<T, T2> {
        ConditionalFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame, F: TaskFrame> TaskFrame for ConditionalFrame<T, F> {
    type Error = ConditionalTaskFrameError<T::Error, F::Error>;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let result = self.predicate.execute(&ctx.0).await;

        if result {
            ctx.emit::<OnTruthyValueEvent>(&()).await; // skipcq: RS-E1015
            return ctx
                .subdivide(&self.frame)
                .await
                .map_err(ConditionalTaskFrameError::PrimaryFailed);
        }

        ctx.emit::<OnFalseyValueEvent>(&()).await; // skipcq: RS-E1015
        let result = ctx.subdivide(&self.fallback).await;
        if self.error_on_false && result.is_ok() {
            return Err(ConditionalTaskFrameError::TaskConditionFail);
        }

        result.map_err(ConditionalTaskFrameError::SecondaryFailed)
    }
}
