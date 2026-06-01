use crate::utils::macros::define_event;
use crate::errors::{DependencyTaskFrameError, TaskDependenciesUnresolved, TaskError};
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::TaskFrame;
use crate::task::{Debug, TaskFrameContext};
use async_trait::async_trait;
use typed_builder::TypedBuilder;

#[async_trait]
pub trait DependentFailBehavior: Send + Sync {
    async fn execute(&self) -> Result<(), Box<dyn TaskError>>;
}

#[derive(Default, Clone, Copy)]
pub struct DependentFailureOnFail;

#[async_trait]
impl DependentFailBehavior for DependentFailureOnFail {
    async fn execute(&self) -> Result<(), Box<dyn TaskError>> {
        Err(Box::new(TaskDependenciesUnresolved))
    }
}

#[derive(Default, Clone, Copy)]
pub struct DependentSuccessOnFail;

#[async_trait]
impl DependentFailBehavior for DependentSuccessOnFail {
    async fn execute(&self) -> Result<(), Box<dyn TaskError>> {
        Ok(())
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    frame: T,

    dependency: FrameDependency,

    #[builder(
        default = Box::new(DependentFailureOnFail),
        setter(transform = |ts: impl DependentFailBehavior + 'static| Box::new(ts) as Box<dyn DependentFailBehavior>)
    )]
    dependent_behaviour: Box<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            dependency: config.dependency,
            dependent_behaviour: config.dependent_behaviour,
        }
    }
}

define_event!(OnDependencyValidation, (&'a FrameDependency, bool));

pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: T,
    dependency: FrameDependency,
    dependent_behaviour: Box<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    type Error = DependencyTaskFrameError<T::Error>;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let is_resolved = self.dependency.is_resolved().await;

        ctx.emit::<OnDependencyValidation>(&(&self.dependency, is_resolved)).await;
        if !is_resolved {
            return self
                .dependent_behaviour
                .execute()
                .await
                .map_err(DependencyTaskFrameError::DependenciesInvalidated);
        }

        self.frame.execute(&ctx, args).await.map_err(DependencyTaskFrameError::Inner)
    }
}
