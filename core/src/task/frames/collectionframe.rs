use crate::errors::TaskError;
use crate::prelude::TaskHookEvent;
use crate::task::{ErasedTaskFrame, TaskFrameContext};
#[allow(unused_imports)]
use crate::task::{RestrictTaskFrameContext, TaskFrame};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

/* === SELECTOR CODE ===
   #[async_trait]
   pub trait SelectFrameAccessor: Send + Sync {
       async fn select(&self, ctx: &TaskFrameContext) -> usize;
   }

   let idx = self.accessor.select(ctx).await;
   if let Some(frame) = self.frames.get(idx) {
       ctx.emit::<OnTaskFrameSelection>(&(idx, frame.clone()))
           .await;
       return ctx.subdivide(frame.clone()).await;
   }

   Err(Arc::new(StandardCoreErrorsCG::TaskIndexOutOfBounds(
       idx,
       "SelectTaskFrame".to_owned(),
       self.frames.len(),
   )))
*/

/* === PARALLEL CODE ===
       let mut js = tokio::task::JoinSet::new();

       for frame in self.tasks.iter() {
           let frame_clone = frame.clone();
           let ctx_clone = ctx.clone();
           js.spawn(async move {
               ctx_clone.emit::<OnChildTaskFrameStart>(&()).await; // skipcq: RS-E1015
               let result = ctx_clone.subdivide(frame_clone.clone()).await;
               ctx_clone
                   .emit::<OnChildTaskFrameEnd>(&result.clone().err())
                   .await; // skipcq: RS-E1015
               result
           });
       }

       while let Some(result) = js.join_next().await {
           let Ok(k) = result else { continue };

           match self.policy.should_quit(k.err()).await {
               ConsensusGTFE::SkipResult => continue,
               ConsensusGTFE::ReturnSuccess => break,
               ConsensusGTFE::ReturnError(err) => return Err(err),
           }
       }

       Ok(())
*/

#[derive(Debug)]
pub struct CollectionTaskError {
    index: usize,
    error: Box<dyn TaskError>,
}

impl CollectionTaskError {
    pub fn new(index: usize, error: Box<dyn TaskError>) -> Self {
        Self { index, error }
    }
}

impl Display for CollectionTaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} inside CollectionTaskFrame at index {}",
            &self.error, self.index
        ))
    }
}

#[async_trait]
pub trait CollectionExecStrategy: Send + Sync + Sized + 'static {
    async fn execute(
        &self,
        handle: CollectionTaskFrameHandle<'_, Self>,
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error>;
}

#[derive(Default, Copy, Clone)]
pub struct SequentialExecStrategy;

#[async_trait]
impl CollectionExecStrategy for SequentialExecStrategy {
    async fn execute(
        &self, handle: CollectionTaskFrameHandle<'_, Self>
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error> {
        for idx in 0..handle.length() {
            handle.run(idx).await
                .map_err(|x| CollectionTaskError::new(idx, x))?;
        }
        Ok(())
    }
}

pub enum ConsensusGTFE<T: Error + Send + Sync + 'static> {
    SkipResult,
    ReturnError(T),
    ReturnSuccess,
}

#[async_trait]
pub trait CollectionExecPolicy<T: Error + Send + Sync + 'static>: Send + Sync {
    async fn should_quit(&self, result: Option<T>) -> ConsensusGTFE<T>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GroupedTaskFramesQuitOnSuccess;

#[async_trait]
impl<T: Error + Send + Sync + 'static> CollectionExecPolicy<T> for GroupedTaskFramesQuitOnSuccess {
    async fn should_quit(&self, result: Option<T>) -> ConsensusGTFE<T> {
        match result {
            None => ConsensusGTFE::ReturnSuccess,
            Some(_) => ConsensusGTFE::SkipResult,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GroupedTaskFramesQuitOnFailure;

#[async_trait]
impl<T: Error + Send + Sync + 'static> CollectionExecPolicy<T> for GroupedTaskFramesQuitOnFailure {
    async fn should_quit(&self, result: Option<T>) -> ConsensusGTFE<T> {
        match result {
            None => ConsensusGTFE::SkipResult,
            Some(err) => ConsensusGTFE::ReturnError(err),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GroupedTaskFramesSilent;

#[async_trait]
impl<T: Error + Send + Sync + 'static> CollectionExecPolicy<T> for GroupedTaskFramesSilent {
    async fn should_quit(&self, _result: Option<T>) -> ConsensusGTFE<T> {
        ConsensusGTFE::SkipResult
    }
}

define_event!(OnChildTaskFrameStart, ());

define_event!(OnChildTaskFrameEnd, Option<&'a dyn TaskError>);

define_event_group!(
    ChildTaskFrameEvents,
    OnChildTaskFrameStart,
    OnChildTaskFrameEnd
);

pub struct CollectionTaskFrame<T: CollectionExecStrategy> {
    taskframes: Vec<Arc<dyn ErasedTaskFrame>>,
    strategy: T,
}

pub struct CollectionTaskFrameHandle<'a, T: CollectionExecStrategy> {
    collection: &'a CollectionTaskFrame<T>,
    ctx: &'a TaskFrameContext<'a>,
}

impl<'a, T: CollectionExecStrategy> CollectionTaskFrameHandle<'a, T> {
    pub async fn run(&self, idx: usize) -> Result<(), Box<dyn TaskError>> {
        let taskframe = self.collection.taskframes[idx].as_ref();
        self.ctx.emit::<OnChildTaskFrameStart>(&()).await; // skipcq: RS-E1015
        let result = self.ctx.erased_subdivide(taskframe).await;
        match result {
            Ok(()) => {
                self.ctx
                    .emit::<OnChildTaskFrameEnd>(&None)
                    .await;
                Ok(())
            }

            Err(err) => {
                self.ctx
                    .emit::<OnChildTaskFrameEnd>(&Some(err.as_ref()))
                    .await;

                Err(err)
            },
        }
    }

    pub fn get(&self, idx: usize) -> Option<&dyn ErasedTaskFrame> {
        self.collection.taskframes.get(idx).map(Arc::as_ref)
    }

    pub fn length(&self) -> usize {
        self.collection.taskframes.len()
    }
}

impl<'a, T: CollectionExecStrategy> Deref for CollectionTaskFrameHandle<'a, T> {
    type Target = RestrictTaskFrameContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx.0
    }
}

#[async_trait]
impl<T: CollectionExecStrategy> TaskFrame for CollectionTaskFrame<T> {
    type Error = CollectionTaskError;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        let handle = CollectionTaskFrameHandle {
            collection: self,
            ctx,
        };

        self.strategy.execute(handle).await
    }
}
