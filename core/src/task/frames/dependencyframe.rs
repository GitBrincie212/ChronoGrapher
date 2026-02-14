use std::error::Error;
use crate::define_event;
use crate::errors::{DependencyTaskFrameError, StandardCoreErrorsCG};
use crate::task::{Debug, TaskFrameContext};
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::{Arc, TaskFrame};
use async_trait::async_trait;
use typed_builder::TypedBuilder;

#[async_trait]
pub trait DependentFailBehavior: Send + Sync {
    async fn execute(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
}

#[derive(Default, Clone, Copy)]
pub struct DependentFailureOnFail;

#[async_trait]
impl DependentFailBehavior for DependentFailureOnFail {
    async fn execute(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        Err(Box::new(StandardCoreErrorsCG::TaskDependenciesUnresolved))
    }
}

#[derive(Default, Clone, Copy)]
pub struct DependentSuccessOnFail;

#[async_trait]
impl DependentFailBehavior for DependentSuccessOnFail {
    async fn execute(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        Ok(())
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    frame: T,

    dependencies: Vec<Arc<dyn FrameDependency>>,

    #[builder(
        default = Arc::new(DependentFailureOnFail),
        setter(transform = |ts: impl DependentFailBehavior + 'static| Arc::new(ts) as Arc<dyn DependentFailBehavior>)
    )]
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            dependencies: config.dependencies,
            dependent_behaviour: config.dependent_behaviour,
        }
    }
}

define_event!(
    OnDependencyValidation, (Arc<dyn FrameDependency>, bool)
);

pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: T,
    dependencies: Vec<Arc<dyn FrameDependency>>,
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    type Error = DependencyTaskFrameError<T::Error>;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let mut js = tokio::task::JoinSet::new();

        for dep in &self.dependencies {
            let dep = dep.clone();
            js.spawn(async move {
                (dep.is_resolved().await, dep)
            });
        }

        let mut is_resolved = true;
        while let Some(result) = js.join_next().await {
            match result {
                Ok((res, dep)) => {
                    ctx.emit::<OnDependencyValidation>(&(dep, res)).await;
                    if !res {
                        is_resolved = false;
                        js.abort_all();
                        break;
                    }
                }

                Err(_) => {
                    is_resolved = false;
                    js.abort_all();
                    break;
                }
            }
        }

        if !is_resolved {
            todo!()
            // return self.dependent_behaviour.execute().await.map_err(DependencyTaskFrameError::new);
        }

        ctx.subdivide(&self.frame).await.map_err(DependencyTaskFrameError::new)
    }
}
