use std::marker::PhantomData;
use crate::utils::macros::define_event;
use crate::errors::TaskError;
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::TaskFrame;
use crate::task::{Debug, TaskFrameContext};
use typed_builder::TypedBuilder;

pub trait DefaultDependencyError: TaskError {
    fn default_dependency_error() -> Self;
}

pub trait DependencyUnresolve<T: TaskError>: Send + Sync {
    fn execute(&self) -> Result<(), T>;
}

pub struct DependencyUnresolveFail<T: TaskError>(PhantomData<T>);

impl<T: TaskError> Default for DependencyUnresolveFail<T> {
    fn default() -> Self {
        DependencyUnresolveFail(PhantomData)
    }
}

impl<T: TaskError> Clone for DependencyUnresolveFail<T> {
    fn clone(&self) -> Self {
        DependencyUnresolveFail(PhantomData)
    }
}

impl<T: DefaultDependencyError> DependencyUnresolve<T> for DependencyUnresolveFail<T> {
    fn execute(&self) -> Result<(), T> {
        Err(T::default_dependency_error())
    }
}

pub struct DependencyUnresolveSkip<T: TaskError>(PhantomData<T>);

impl<T: TaskError> Default for DependencyUnresolveSkip<T> {
    fn default() -> Self {
        DependencyUnresolveSkip(PhantomData)
    }
}

impl<T: TaskError> Clone for DependencyUnresolveSkip<T> {
    fn clone(&self) -> Self {
        DependencyUnresolveSkip(PhantomData)
    }
}

impl<T: TaskError> DependencyUnresolve<T> for DependencyUnresolveSkip<T> {
    fn execute(&self) -> Result<(), T> {
        Ok(())
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    frame: T,

    dependency: FrameDependency,

    #[builder(
        default = Box::new(DependencyUnresolveSkip::<T::Error>::default()),
        setter(transform = |ts: impl DependencyUnresolve<T::Error> + 'static| Box::new(ts) as Box<dyn DependencyUnresolve<_>>)
    )]
    unresolve: Box<dyn DependencyUnresolve<T::Error>>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: config.frame,
            dependency: config.dependency,
            unresolve: config.unresolve,
        }
    }
}

define_event!(OnDependencyValidation, (&'a FrameDependency, bool));

pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: T,
    dependency: FrameDependency,
    unresolve: Box<dyn DependencyUnresolve<T::Error>>,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Workflow = Self;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        let is_resolved = self.dependency.is_resolved().await;

        ctx.emit::<OnDependencyValidation>(&(&self.dependency, is_resolved)).await;
        if !is_resolved {
            return self.unresolve.execute()
        }

        self.frame.execute(&ctx, args).await
    }
}
