use crate::errors::{StandardCoreErrorsCG, TaskError};
use crate::prelude::TaskHookEvent;
use crate::task::{ErasedTaskFrame, RestrictTaskFrameContext, TaskFrame, TaskFrameContext};
use crate::{define_event, define_event_group};
use async_trait::async_trait;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

///
#[derive(Debug)]
pub struct CollectionTaskError {
    index: usize,
    error: Box<dyn TaskError>,
}

impl CollectionTaskError {
    pub fn new(index: usize, error: Box<dyn TaskError>) -> Self {
        Self { index, error }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn inner(&self) -> &dyn TaskError {
        self.error.as_ref()
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

impl Error for CollectionTaskError {}
/// Defines how a [`CollectionTaskFrame`] executes its taskframes, controlling
/// order, selection, concurrency, and early termination while preserving the
/// [`TaskFrame`] contract.
///
/// Operates via a [`CollectionTaskFrameHandle`], providing access to the taskframe
/// collection, lifecycle hooks, indexed execution, and context-based subdivision.
///
/// Implementations decide which frames to run, how to schedule them
/// (sequential, parallel, or custom), and how child `TaskError`s map to
/// `CollectionTaskError`, optionally using an execution policy.
///
/// [`CollectionExecStrategy`] is one of the four systems (manages the orchestration part) for
/// executing upon a collection, with [`CollectionTaskFrame`] being for storage,
/// [`CollectionTaskFrameHandle`] for context and optionally [`CollectionExecPolicy`] for enforcing
/// termination.
///
/// # Semantics
/// Acts as a behavioral controller over a [`CollectionTaskFrame`]. May execute
/// zero, some, or all frames, in any order, including concurrently.
///
/// # Required Method(s)
/// ## execute
/// [`CollectionExecStrategy`] **requires** users to implement the `execute` by any `CollectionExecStrategy`.
/// It defines how the strategy executes the taskframes in a [`CollectionTaskFrame`].
///
/// It accepts ``CollectionTaskFrameHandle<'_, Self>`` as an argument, providing access to
/// the [`TaskFrame`] collection, context around the execution, lifecycle hooks and
/// safe-indexed execution.
///
/// # Object Safety / Dynamic Dispatching
/// [`CollectionExecStrategy`] is **not object-safe** and cannot be used as a
/// trait object (e.g., `Box<dyn CollectionExecStrategy>`).
///
/// Reasons:
/// The reason for this is due [`Sized`] requirement, the fact the ``execute`` method takes ``CollectionTaskFrameHandle<'_, Self>`` which references Self in its arguments.
///
/// Finally, the return type ``<CollectionTaskFrame<Self> as TaskFrame>::Error`` which depends on
/// Self.
///
/// # Implementations
/// The main implementors of [`CollectionExecStrategy`] are:
/// - [`SequentialExecStrategy`] - Executes taskframes sequentially in index order.
/// - [`ParallelExecStrategy`] - Executes all taskframes concurrently.
/// - [`SelectionExecStrategy`] - Executes a single taskframe selected at runtime.
///
/// # Example(s)
/// **1. Sequential execution (default failure policy)**
///```rust
/// use std::sync::Arc;
///
/// let frame = CollectionTaskFrame::sequential(vec![
///     Arc::new(task_a),
///     Arc::new(task_b),
///     Arc::new(task_c),
/// ]);
///```
/// This runs taskframes in order and stops on the first failure.
///
/// **2. Parallel execution with custom policy**
///```rust
/// use std::sync::Arc;
///
/// let frame = CollectionTaskFrame::parallel(
///     vec![
///         Arc::new(task_a),
///         Arc::new(task_b),
///         Arc::new(task_c),
///     ],
///     GroupedTaskFramesQuitOnSuccess,
/// );
///```
/// This runs all taskframes concurrently and terminates once one succeeds.
/// # See Also
/// - [`CollectionTaskFrame`] - The collection which [`CollectionExecStrategy`] acts upon.
/// - [`CollectionTaskFrameHandle`] - Useful context around the collection for [`CollectionExecStrategy`] to use.
/// - [`TaskFrame`] - The system which [`CollectionExecStrategy`] is used in.
/// - [`CollectionExecPolicy`] - An optional termination policy which can be embedded to [`CollectionExecStrategy`].
/// - [`ConsensusGTFE`] - An enum from the [`CollectionExecPolicy`] determining the results.
/// - [`SequentialExecStrategy`] - An implementation of [`CollectionExecStrategy`] for sequential execution.
/// - [`ParallelExecStrategy`] - An implementation of [`CollectionExecStrategy`] for concurrent execution.
/// - [`SelectionExecStrategy`] - An implementation of [`CollectionExecStrategy`] for selective execution.
#[async_trait]
pub trait CollectionExecStrategy: Send + Sync + Sized + 'static {
    async fn execute(
        &self,
        handle: CollectionTaskFrameHandle<'_, Self>,
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error>;
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

#[derive(Debug, Default, Copy, Clone)]
pub struct SequentialExecStrategy<P = GroupedTaskFramesQuitOnFailure> {
    policy: P,
}

impl<P> SequentialExecStrategy<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl<P: CollectionExecPolicy<CollectionTaskError> + Send + Sync + 'static> CollectionExecStrategy
    for SequentialExecStrategy<P>
{
    async fn execute(
        &self,
        handle: CollectionTaskFrameHandle<'_, Self>,
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error> {
        for idx in 0..handle.length() {
            let result = handle
                .execute(idx)
                .await
                .err()
                .map(|err| CollectionTaskError::new(idx, err));

            match self.policy.should_quit(result).await {
                ConsensusGTFE::SkipResult => continue,
                ConsensusGTFE::ReturnSuccess => return Ok(()),
                ConsensusGTFE::ReturnError(err) => return Err(err),
            }
        }

        Ok(())
    }
}

pub struct ParallelExecStrategy<P = GroupedTaskFramesQuitOnFailure> {
    policy: P,
}

impl<P> ParallelExecStrategy<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }
}

impl Default for ParallelExecStrategy<GroupedTaskFramesQuitOnFailure> {
    fn default() -> Self {
        Self {
            policy: GroupedTaskFramesQuitOnFailure,
        }
    }
}

#[async_trait]
impl<P: CollectionExecPolicy<CollectionTaskError> + Send + Sync + 'static> CollectionExecStrategy
    for ParallelExecStrategy<P>
{
    async fn execute(
        &self,
        handle: CollectionTaskFrameHandle<'_, Self>,
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error> {
        if handle.length() == 0 {
            return Ok(());
        }

        let mut js = tokio::task::JoinSet::new();
        for idx in 0..handle.length() {
            let hooks_container = handle.ctx.0.hooks_container.clone();
            let frame = handle.collection.taskframes[idx].clone();
            let depth = handle.ctx.0.depth + 1;
            js.spawn(async move {
                let child_ctx = TaskFrameContext(RestrictTaskFrameContext {
                    hooks_container,
                    depth,
                    frame: frame.as_ref(),
                });

                child_ctx
                    .emit::<OnChildTaskFrameStart>(&(idx, frame.as_ref()))
                    .await;
                let result = child_ctx.erased_subdivide(frame.as_ref()).await;
                match result {
                    Ok(()) => child_ctx.emit::<OnChildTaskFrameEnd>(&None).await,
                    Err(ref err) => {
                        child_ctx
                            .emit::<OnChildTaskFrameEnd>(&Some(err.as_ref()))
                            .await
                    }
                }

                (idx, result)
            });
        }

        while let Some(joined) = js.join_next().await {
            let Ok((idx, result)) = joined else {
                continue;
            };
            let result = result.err().map(|err| CollectionTaskError::new(idx, err));

            match self.policy.should_quit(result).await {
                ConsensusGTFE::SkipResult => continue,
                ConsensusGTFE::ReturnSuccess => return Ok(()),
                ConsensusGTFE::ReturnError(err) => return Err(err),
            }
        }

        Ok(())
    }
}

#[async_trait]
pub trait SelectFrameAccessor: Send + Sync + 'static {
    async fn select(&self, ctx: &RestrictTaskFrameContext<'_>) -> usize;
}

#[async_trait]
impl<F, Fut> SelectFrameAccessor for F
where
    F: Fn(&RestrictTaskFrameContext<'_>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = usize> + Send,
{
    async fn select(&self, ctx: &RestrictTaskFrameContext<'_>) -> usize {
        self(ctx).await
    }
}

pub struct SelectionExecStrategy<S: SelectFrameAccessor> {
    accessor: S,
}

impl<S: SelectFrameAccessor> SelectionExecStrategy<S> {
    pub fn new(accessor: S) -> Self {
        Self { accessor }
    }
}

#[async_trait]
impl<S: SelectFrameAccessor> CollectionExecStrategy for SelectionExecStrategy<S> {
    async fn execute(
        &self,
        handle: CollectionTaskFrameHandle<'_, Self>,
    ) -> Result<(), <CollectionTaskFrame<Self> as TaskFrame>::Error> {
        let idx = self.accessor.select(handle.deref()).await;
        if handle.get(idx).is_none() {
            return Err(CollectionTaskError::new(
                idx,
                Box::new(StandardCoreErrorsCG::TaskIndexOutOfBounds(
                    idx,
                    "SelectionExecStrategy".to_owned(),
                    handle.length(),
                )) as Box<dyn TaskError>,
            ));
        };

        handle
            .execute(idx)
            .await
            .map_err(|err| CollectionTaskError::new(idx, err))
    }
}

define_event!(OnChildTaskFrameStart, (usize, &'a dyn ErasedTaskFrame));
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

impl<T: CollectionExecStrategy> CollectionTaskFrame<T> {
    pub fn new(taskframes: Vec<Arc<dyn ErasedTaskFrame>>, strategy: T) -> Self {
        Self {
            taskframes,
            strategy,
        }
    }

    pub fn strategy(&self) -> &T {
        &self.strategy
    }

    pub fn taskframes(&self) -> &[Arc<dyn ErasedTaskFrame>] {
        &self.taskframes
    }
}

impl CollectionTaskFrame<SequentialExecStrategy<GroupedTaskFramesQuitOnFailure>> {
    pub fn sequential(taskframes: Vec<Arc<dyn ErasedTaskFrame>>) -> Self {
        Self {
            taskframes,
            strategy: SequentialExecStrategy::default(),
        }
    }
}

impl<P: CollectionExecPolicy<CollectionTaskError> + Send + Sync + 'static>
    CollectionTaskFrame<ParallelExecStrategy<P>>
{
    pub fn parallel(taskframes: Vec<Arc<dyn ErasedTaskFrame>>, policy: P) -> Self {
        Self {
            taskframes,
            strategy: ParallelExecStrategy::new(policy),
        }
    }
}

impl<S: SelectFrameAccessor> CollectionTaskFrame<SelectionExecStrategy<S>> {
    pub fn selection(taskframes: Vec<Arc<dyn ErasedTaskFrame>>, accessor: S) -> Self {
        Self {
            taskframes,
            strategy: SelectionExecStrategy::new(accessor),
        }
    }
}

pub struct CollectionTaskFrameHandle<'a, T: CollectionExecStrategy> {
    collection: &'a CollectionTaskFrame<T>,
    ctx: &'a TaskFrameContext<'a>,
}

impl<'a, T: CollectionExecStrategy> CollectionTaskFrameHandle<'a, T> {
    pub async fn execute(&self, idx: usize) -> Result<(), Box<dyn TaskError>> {
        let Some(taskframe) = self.collection.taskframes.get(idx).map(Arc::as_ref) else {
            return Err(Box::new(StandardCoreErrorsCG::TaskIndexOutOfBounds(
                idx,
                "CollectionTaskFrameHandle::execute".to_owned(),
                self.length(),
            )) as Box<dyn TaskError>);
        };

        self.ctx
            .emit::<OnChildTaskFrameStart>(&(idx, taskframe))
            .await;
        let result = self.ctx.erased_subdivide(taskframe).await;
        match result {
            Ok(()) => {
                self.ctx.emit::<OnChildTaskFrameEnd>(&None).await;
                Ok(())
            }

            Err(err) => {
                self.ctx
                    .emit::<OnChildTaskFrameEnd>(&Some(err.as_ref()))
                    .await;
                Err(err)
            }
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
