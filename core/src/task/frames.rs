pub mod conditionframe; // skipcq: RS-D1001

pub mod dependencyframe; // skipcq: RS-D1001

pub mod fallbackframe; // skipcq: RS-D1001

pub mod noopframe; // skipcq: RS-D1001

pub mod parallelframe; // skipcq: RS-D1001

pub mod retryframe; // skipcq: RS-D1001

pub mod selectframe; // skipcq: RS-D1001

pub mod sequentialframe; // skipcq: RS-D1001

pub mod timeoutframe; // skipcq: RS-D1001

pub mod misc; // skipcq: RS-D1001

pub mod delayframe; // skipcq: RS-D1001

pub mod dynamicframe; // skipcq: RS-D1001

use crate::task::DashMap;
use crate::task::{ErasedTask, TaskHook, TaskHookContainer, TaskHookEvent};
use async_trait::async_trait;
pub use conditionframe::*;
pub use delayframe::*;
pub use dependencyframe::*;
pub use fallbackframe::*;
pub use misc::*;
pub use noopframe::*;
pub use parallelframe::*;
pub use retryframe::*;
pub use selectframe::*;
pub use sequentialframe::*;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
pub use timeoutframe::*;

/// A task-related error (i.e. A task failure)
pub type TaskError = Arc<dyn Debug + Send + Sync>;

/// [`TaskContext`] is a mechanism that wraps commonly needed information
/// inside it which can be accessed by [`TaskFrame`], it essentially wraps
/// - metadata: [`TaskMetadata`]
/// - emitter: [`TaskEventEmitter`]
/// - priority: [`TaskPriority`]
/// - runs: ``u64``
/// - debug_label: ``String``
/// - max_runs: ``Option<NonZeroU64>``
///
/// All of them fetched in [`Task`]. The [`TaskContext`] object can also
/// be restricted to disallow event emission, this is useful to ensure
/// no other source can emit naively any [`TaskEvent`]
///
/// # Constructor(s)
/// There are no public constructors as this context's constructor is sealed
///
/// # Task Implementation(s)
/// The [`TaskContext`] only implements [`Clone`] and [`Debug`] as there is no other use for it.
/// Where for [`Debug`] it outputs all the fields except the event emitter (it holds no data, so no
/// point in recording that)
///
/// # See Also
/// - [`Task`]
/// - [`TaskFrame`]
/// - [`TaskMetadata`]
/// - [`TaskPriority`]
/// - [`TaskEventEmitter`]
pub struct TaskContext {
    hooks_container: Arc<TaskHookContainer>,
    depth: u64,
    frame: Arc<dyn TaskFrame>,
    shared_data: Arc<DashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl Clone for TaskContext {
    fn clone(&self) -> Self {
        Self {
            hooks_container: self.hooks_container.clone(),
            depth: self.depth,
            frame: self.frame.clone(),
            shared_data: self.shared_data.clone(),
        }
    }
}

pub struct SharedHandle<T> {
    data: Arc<RwLock<T>>,
}

impl<T> SharedHandle<T> {
    fn owner(data: Arc<RwLock<T>>) -> Self {
        Self { data }
    }
    fn existing(data: Arc<RwLock<T>>) -> Self {
        Self { data }
    }

    pub fn read(&self) -> impl Deref<Target = T> + '_ {
        self.data.read().unwrap()
    }

    pub fn write(&self) -> impl DerefMut<Target = T> + '_ {
        self.data.write().unwrap()
    }
}

impl TaskContext {
    /// Constructs / Creates a new [`TaskContext`] instance based for use inside [`TaskFrame`],
    /// unlike most constructors, this mechanism is sealed and accessible only in the library's
    /// internal code
    ///
    /// # Argument(s)
    /// This method accepts 2 arguments, those being [`Task`] and [`TaskEventEmitter`] wrapped in an
    /// ``Arc<T>``, the former is for retrieving most of the data. While the latter is not only to prove
    /// that this construction is made internally on [`Task`] but also to share it with [`TaskFrame`]
    /// to emit its own events
    ///
    /// # Returns
    /// The constructed instance to be used
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskEventEmitter`]
    /// - [`TaskContext`]
    pub(crate) fn new(task: &ErasedTask) -> Self {
        Self {
            hooks_container: task.hooks.clone(),
            depth: 0,
            frame: task.frame.clone(),
            shared_data: Arc::new(DashMap::new()),
        }
    }

    pub(crate) fn subdivided_ctx(&self, frame: Arc<dyn TaskFrame>) -> Self {
        Self {
            hooks_container: self.hooks_container.clone(),
            frame: frame.clone(),
            depth: self.depth + 1,
            shared_data: self.shared_data.clone(),
        }
    }

    pub async fn subdivide(&self, frame: Arc<dyn TaskFrame>) -> Result<(), TaskError> {
        let child_ctx = self.subdivided_ctx(frame.clone());
        frame.execute(&child_ctx).await
    }

    pub fn frame(&self) -> &dyn TaskFrame {
        &self.frame
    }

    /// Emits an event to relevant [`TaskHook(s)`] that have subscribed to it
    ///
    /// # Arguments
    /// The method accepts one argument, that being the payload to supply
    /// from the generic ``E`` where it is the [`TaskHookEvent`] type
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    pub async fn emit<E: TaskHookEvent>(&self, payload: &E::Payload) {
        self.hooks_container.emit::<E>(self, payload).await;
    }

    /// Attaches an **Ephemeral** [`TaskHook`] to a specific [`TaskHookEvent`]. This is a much more
    /// ergonomic method-alias to the relevant [`TaskHookContainer::attach_ephemeral`] method.
    ///
    /// When the program crashes, these TaskHooks do not persist. Depending on the circumstances,
    /// this may not be a wanted behavior, if you can guarantee your TaskHook is persistable,
    /// then [`TaskContext::attach_persistent_hook`] is the ideal method for you
    ///
    /// # Arguments
    /// The method accepts one argument, that being the [`TaskHook`] instance
    /// to supply, which will subscribe to the [`TaskHookEvent`]
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    pub async fn attach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self, hook: Arc<T>) {
        self.hooks_container.attach::<E, T>(self, hook).await;
    }

    /// Detaches a [`TaskHook`] from a specific [`TaskHookEvent`]. This is a much more
    /// ergonomic method-alias to the relevant [`TaskHookContainer::detach`] method
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    pub async fn detach_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) {
        self.hooks_container.detach::<E, T>(self).await;
    }

    /// Gets a [`TaskHook`] instance from a specific [`TaskHookEvent`]. This is a much more
    /// ergonomic method-alias to the relevant [`TaskHookContainer::get`] method
    ///
    /// # Returns
    /// An optional [`TaskHook`] instance, if it doesn't exist ``None`` is returned,
    /// if it does, then it returns ``Some(TaskHook)``
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    pub fn get_hook<E: TaskHookEvent, T: TaskHook<E>>(&self) -> Option<Arc<T>> {
        self.hooks_container.get::<E, T>()
    }

    pub fn shared<T, F>(&self, creator: F) -> SharedHandle<T>
    where
        T: Any + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.shared_data.get(&type_id) {
            return SharedHandle::existing(
                existing.downcast_ref::<Arc<RwLock<T>>>().unwrap().clone(),
            );
        }
        let data = Arc::new(RwLock::new(creator()));
        self.shared_data.insert(type_id, Box::new(data.clone()));

        SharedHandle::owner(data)
    }

    pub async fn shared_async<T, F, Fut>(&self, creator: F) -> SharedHandle<T>
    where
        T: Any + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = T> + Send + 'static,
    {
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.shared_data.get(&type_id) {
            return SharedHandle::existing(
                existing.downcast_ref::<Arc<RwLock<T>>>().unwrap().clone(),
            );
        }
        let data = Arc::new(RwLock::new(creator().await));
        self.shared_data.insert(type_id, Box::new(data.clone()));

        SharedHandle::owner(data)
    }

    pub fn get_shared<T>(&self) -> Option<SharedHandle<T>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.shared_data.get(&type_id).map(|data| {
            SharedHandle::existing(data.downcast_ref::<Arc<RwLock<T>>>().unwrap().clone())
        })
    }
}

/// [`TaskFrame`] represents a unit of work which hosts the actual computation logic for
/// the [`Scheduler`] to invoke, this is a part of the task system
///
/// # Required Method(s)
/// When one implements the [`TaskFrame`] trait, one has to implement [`TaskFrame::execute`] which
/// encapsulates mainly the async execution logic of the [`Task`], the method has as argument
/// a [`TaskContext`], wrapping any essential logic (while also being impossible to create it
/// outside the [`Task`]), the method also returns either `Ok(())` on success, or a `TaskError`
/// on failure. The method also handles the emission of local task frame events
/// (learn more about in [`TaskEvent`])
///
/// # Usage Notes
/// This is one of many components which are combined to form a task, other components are needed
/// to fuse them to a task. By itself it is not as useful, as such combine it to form a task with other
/// composites
///
/// [`TaskFrame`] can be decorated with other task unit implementations to expand the behavior, such
/// as adding retry mechanism via [`RetryFrame`], adding timeout via [`TimeoutFrame`]... etc. Some
/// examples (from simple to complex) include:
/// - **RetriableTaskFrame<T>** Executes a task frame ``T``, when the task frame succeeds at some point
/// it stops and returns the success results. Otherwise, if it fails, it retires it ``N`` times (controlled
/// by the developer) til it succeeds, or it reaches this threshold with a specified backoff retry strategy
///
/// - **RetriableTaskFrame<DependencyTaskFrame<T>>** Executes a task frame ``T``, if all of its
/// dependencies are resolved and ``T`` succeeds at some point (both have to be satisfied), it stops
/// and returns the success results. Otherwise, if it fails, it retires it ``N`` (controlled by the
/// developer) til it succeeds with a specified backoff retry strategy
///
/// - **``RetriableTaskFrame<TimeoutTaskFrame<T>>``**: Execute task frame `T`, when the
/// task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
/// then finish, otherwise if it exceeds its maximum duration or if the task frame failed then
/// abort it and retry it again, repeating this process `N` times (can be controlled by the developer)
/// with a specified backoff retry strategy
///
/// - **``FallbackTaskFrame<TimeoutTaskFrame<T1>, RetriableTaskFrame<T2>>``**: Execute task frame `T1`,
/// when the task frame succeeds within a maximum duration of `D` (can be controlled by the developer)
/// then finish, otherwise if it either fails or it reaches its maximum duration then execute
/// task frame `T2` (as a fallback), try/retry executing this task frame for `N` times (can be
/// controlled by the developer) with a delay per retry of `d` (can be controlled by the developer),
/// regardless if it succeeds at some time or fails entirely, return the result back
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`Task`] struct
///
/// # Trait Implementation(s)
/// There are various implementations for [`TaskFrame`] present in the library, each
/// doing their own part. Some noteworthy mentions include
/// - [`RetriableTaskFrame`] Retries a task frame a specified number of times with a delay
/// per retry based on a supplied [`RetryBackoffStrategy`] (more info on the docs of it)
///
/// - [`TimeoutTaskFrame`] Runs a task frame for a specified duration, if the countdown reaches
/// zero, then it halts the task and returns a timeout error (more info on the docs of it)
///
/// - [`DependencyTaskFrame`] Before running a task frame, it checks if its dependencies are resolved,
/// if they are then it runs, otherwise it errors out with dependencies unresolved
///
/// - [`FallbackTaskFrame`] Attempts to run a task frame, if it fails, then a fallback secondary task frame
/// takes its place which the result of this fallback task frame is returned from the secondary (more
/// info on the docs of it)
///
/// - [`NoOperationTaskFrame`] Effectively acts as a placeholder and does nothing useful, in
/// some circumstances where task frames may be optional to supply (more info on the docs of it)
///
/// It is advised to check the submodules of the task module to see more of them in action
///
/// # See Also
/// - [`TaskFrame::execute`]
/// - [`RetriableTaskFrame`]
/// - [`TimeoutTaskFrame`]
/// - [`DependencyTaskFrame`]
/// - [`FallbackTaskFrame`]
/// - [`NoOperationTaskFrame`]
/// - [`ConditionalTaskFrame`]
/// - [`SelectTaskFrame`]
/// - [`Task`]
#[async_trait]
pub trait TaskFrame: 'static + Send + Sync {
    /// The execution logic of the [`TaskFrame`] and subsequentially [`Task`]
    ///
    /// # Argument(s)
    /// This method accepts one argument, that being ``ctx`` which is the context object,
    /// this context object is private and cannot be created by outside parties, but only in
    /// [`Task`]. The context wraps common information for the [`TaskFrame`] to access and pass
    /// on to other child [`TaskFrame`]s
    ///
    /// # Returns
    /// A ``Result<(), TaskError>`` which on success returns ``Ok(())`` (i.e. No result) and on
    /// failure it returns a ``Err(TaskError)`` indicating what went wrong on the [`TaskFrame`]
    ///
    /// # See Also
    /// - [`Task`]
    /// - [`TaskContext`]
    /// - [`TaskFrame`]
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError>;
}

#[async_trait]
impl<S> TaskFrame for S
where
    S: Deref + Send + Sync + 'static,
    S::Target: TaskFrame,
{
    async fn execute(&self, task: &TaskContext) -> Result<(), TaskError> {
        self.deref().execute(task).await
    }
}
