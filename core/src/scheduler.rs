pub mod clock; // skipcq: RS-D1001
pub mod engine; // skipcq: RS-D1001
pub mod task_dispatcher; // skipcq: RS-D1001
pub mod task_store; // skipcq: RS-D1001

use crate::errors::TaskError;
use crate::prelude::TaskHook;
use crate::scheduler::clock::*;
use crate::scheduler::engine::default::SchedulerHandleInstructions;
use crate::scheduler::engine::{DefaultSchedulerEngine, SchedulerEngine, SchedulerHandlePayload};
use crate::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use crate::scheduler::task_store::EphemeralSchedulerTaskStore;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{ErasedTask, Task, TaskFrame, TaskTrigger};
use crate::utils::{DefaultTaskID, TaskIdentifier};
use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::join;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

/// The default scheduler type.
///
/// # Usage
/// Use the [`DefaultScheduler`] type as the default implementation of the [`Scheduler`] type.
/// Use the [`DefaultSchedulerConfig`] type as the default implementation of the [`SchedulerConfig`] type.
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::DefaultScheduler;
///
/// let scheduler = DefaultScheduler::<Box<dyn std::error::Error + Send + Sync>>::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// ```
///
/// In the example, we use the [`DefaultScheduler`] type as the default implementation of the [`Scheduler`] type.
pub type DefaultScheduler<E> = Scheduler<DefaultSchedulerConfig<E>>;

/// The default scheduler type with the [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) error type.
///
/// # Usage
/// Use the [`DefaultAnyhowScheduler`] type as the default implementation of the [`Scheduler`] type. The [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) error type is used as the error type for the scheduler allowing users to have ergonomic error propagation.
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::DefaultAnyhowScheduler;
///
/// let scheduler = DefaultAnyhowScheduler::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// ```
///
/// In the example, we use the [`DefaultAnyhowScheduler`](crate::scheduler::DefaultAnyhowScheduler) type as the default implementation of the [`Scheduler`](crate::scheduler::Scheduler) type.
#[cfg(feature = "anyhow")]
pub type DefaultAnyhowScheduler = DefaultScheduler<anyhow::Error>;

/// The default scheduler type with the [`eyre`](https://docs.rs/eyre/latest/eyre/) error type.
///
/// # Usage
/// Use the [`DefaultEyreScheduler`](crate::scheduler::DefaultEyreScheduler) type as the default implementation of the [`Scheduler`](crate::scheduler::Scheduler) type. The [`eyre`](https://docs.rs/eyre/latest/eyre/) error type is used as the error type for the scheduler allowing users to have richer, customizable error reports (often paired with [`color-eyre`](https://docs.rs/color-eyre/latest/color_eyre/) in CLI tooling).
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::DefaultEyreScheduler;
///
/// let scheduler = DefaultEyreScheduler::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// ```
///
/// In the example, we use the [`DefaultEyreScheduler`](crate::scheduler::DefaultEyreScheduler) type as the default implementation of the [`Scheduler`](crate::scheduler::Scheduler) type.
#[cfg(feature = "eyre")]
pub type DefaultEyreScheduler = DefaultScheduler<eyre::Error>;

// TODO: Add more details about the configuration with custom types.
/// The `SchedulerConfig` trait for configuring the scheduler.
///
/// # Usage
/// Implement the [`SchedulerConfig`] trait to configure the scheduler. The scheduler configures the identifier, error type, clock, store, dispatcher, and engine types.
pub trait SchedulerConfig: Sized + 'static {
    type TaskIdentifier: TaskIdentifier;
    type TaskError: TaskError;
    type SchedulerClock: SchedulerClock<Self>;
    type SchedulerTaskStore: SchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher: SchedulerTaskDispatcher<Self>;
    type SchedulerEngine: SchedulerEngine<Self>;
}

/// Default implementation of [`SchedulerConfig`], parameterized by the task error type `E`.
///
/// This type is used as the config for [`DefaultScheduler`]`<E>` (i.e. `Scheduler<DefaultSchedulerConfig<E>>`).
/// You do not construct `DefaultSchedulerConfig` values directly; you obtain a scheduler via
/// [`DefaultScheduler`]`::default()` or [`Scheduler::builder`], which use this config under the hood.
///
/// # Type parameters
/// - `E` - The task error type, must implement [`TaskError`] (and thus
///   `Debug + Display + Send + Sync + Any`). For example: [`StandardCoreErrorsCG`](crate::errors::StandardCoreErrorsCG),
///   or with features `anyhow` / `eyre`: `anyhow::Error` / `eyre::Error`. Note: `Box<dyn std::error::Error + Send + Sync>` does *not* implement `TaskError` (it does not implement `Any`).
///
/// **The default implementation** uses:
/// - **[`DefaultTaskID`]** - task identifier
/// - **[`ProgressiveClock`]** - clock
/// - **[`EphemeralSchedulerTaskStore`]** - in-memory task store
/// - **[`DefaultTaskDispatcher`]** - task dispatcher
/// - **[`DefaultSchedulerEngine`]** - engine
pub struct DefaultSchedulerConfig<E: TaskError>(PhantomData<E>);

/// The default implementation of the [`SchedulerConfig`] trait.
///
/// The default implementation uses the [`DefaultTaskID`] type as the identifier, the [`ProgressiveClock`] type as the clock, the [`EphemeralSchedulerTaskStore`] type as the store, the [`DefaultTaskDispatcher`] type as the dispatcher, and the [`DefaultSchedulerEngine`] type as the engine.
///
/// This type is used as the config for [`DefaultScheduler`]`<E>` (i.e. `Scheduler<DefaultSchedulerConfig<E>>`).
/// You do not construct `DefaultSchedulerConfig` values directly; you obtain a scheduler via
/// [`DefaultScheduler`]`::default()` or [`Scheduler::builder`], which use this config under the hood.
impl<E: TaskError> SchedulerConfig for DefaultSchedulerConfig<E> {
    type TaskIdentifier = DefaultTaskID;
    type TaskError = E;
    type SchedulerClock = ProgressiveClock;
    type SchedulerTaskStore = EphemeralSchedulerTaskStore<Self>;
    type SchedulerTaskDispatcher = DefaultTaskDispatcher<Self>;
    type SchedulerEngine = DefaultSchedulerEngine<Self>;
}

/// The `SchedulerInitConfig` type for the configuration of the scheduler.
///
/// # Usage
/// Use by the [`Scheduler::builder`] method to build a scheduler. The builder method is used to configure the scheduler with the desired types.
/// As the builder method is used to build a scheduler, you do not construct `SchedulerInitConfig` values directly; you obtain a scheduler via `Scheduler::default()`.
#[derive(TypedBuilder)]
#[builder(build_method(into = Scheduler<T>))]
pub struct SchedulerInitConfig<T: SchedulerConfig> {
    dispatcher: T::SchedulerTaskDispatcher,

    store: T::SchedulerTaskStore,

    clock: T::SchedulerClock,

    engine: T::SchedulerEngine,
}

/// The `From` trait implementation for the `SchedulerInitConfig` type.
///
/// # Usage
/// Use the [`From`] trait implementation to convert a [`SchedulerInitConfig`] value into a [`Scheduler`] value.
/// As the builder method is used to build a scheduler, you do not construct `SchedulerInitConfig` values directly; you obtain a scheduler via `Scheduler::default()`.
///
/// # Example
/// Even though you do not construct `SchedulerInitConfig` values directly, you can still convert it into a `Scheduler` value using the `From` trait implementation.
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::SchedulerInitConfig;
///
/// let config = SchedulerInitConfig::<Box<dyn std::error::Error + Send + Sync>>::default();
/// let scheduler = Scheduler::from(config);
/// ```
impl<C: SchedulerConfig> From<SchedulerInitConfig<C>> for Scheduler<C> {
    fn from(config: SchedulerInitConfig<C>) -> Self {
        Self {
            dispatcher: Arc::new(config.dispatcher),
            store: Arc::new(config.store),
            clock: Arc::new(config.clock),
            process: Mutex::new(None),
            engine: Arc::new(config.engine),
            instruction_channel: Mutex::new(None),
        }
    }
}

/// The `Scheduler` type for the scheduler.
///
/// # Usage
/// Use the [`Scheduler`] type to create a scheduler. The scheduler is used to schedule tasks. The scheduler owns a clock, a task store, a task dispatcher, and an engine.
/// The scheduler can be started, stopped, and scheduled tasks.
///
/// # Type parameters
/// - `C` - The scheduler config type, must implement [`SchedulerConfig`] (and thus
///   `Sized + 'static`). For example: [`DefaultSchedulerConfig`].
///
/// # Fields
/// - `clock` - The clock of the scheduler.
/// - `store` - The task store of the scheduler.
/// - `dispatcher` - The task dispatcher of the scheduler.
/// - `engine` - The engine of the scheduler.
/// - `process` - The process of the scheduler.
/// - `instruction_channel` - The instruction channel of the scheduler.
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::Scheduler;
///
/// let scheduler = Scheduler::<Box<dyn std::error::Error + Send + Sync>>::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// ```
pub struct Scheduler<C: SchedulerConfig> {
    clock: Arc<C::SchedulerClock>,
    store: Arc<C::SchedulerTaskStore>,
    dispatcher: Arc<C::SchedulerTaskDispatcher>,
    engine: Arc<C::SchedulerEngine>,
    process: Mutex<Option<JoinHandle<()>>>,
    instruction_channel: Mutex<Option<tokio::sync::mpsc::Sender<SchedulerHandlePayload>>>,
}

/// Implements [`Default`] for [`Scheduler`]`<C>` when the single config type `C` has all of its
/// associated types implementing [`Default`]: `C::SchedulerTaskStore`, `C::SchedulerClock`,
/// `C::SchedulerEngine`, and `C::SchedulerTaskDispatcher`. For example: [`DefaultSchedulerConfig`].
///
/// The returned scheduler is equivalent to [`Scheduler::builder`] with default store, clock,
/// engine, and dispatcher for that config. [`DefaultSchedulerConfig`]`<E>` meets these bounds for
/// any `E: [`TaskError`](crate::errors::TaskError)`, so [`DefaultScheduler`]`<E>` has `Default` in that case.
impl<C> Default for Scheduler<C>
where
    C: SchedulerConfig<
            SchedulerTaskStore: Default,
            SchedulerTaskDispatcher: Default,
            SchedulerEngine: Default,
            SchedulerClock: Default,
            TaskError: TaskError,
        >,
{
    /// The default implementation of the [`Default`] trait for the [`Scheduler`] type.
    ///
    /// Uses the [`Scheduler::builder`] method to create a scheduler with the default config.
    fn default() -> Self {
        Self::builder()
            .store(C::SchedulerTaskStore::default())
            .clock(C::SchedulerClock::default())
            .engine(C::SchedulerEngine::default())
            .dispatcher(C::SchedulerTaskDispatcher::default())
            .build()
    }
}

/// The `SchedulerHandle` type for the scheduler handle.
///
/// # Usage
/// Use the [`SchedulerHandle`] type to create a scheduler handle. The scheduler handle is used to instruct the scheduler to reschedule, halt, block, or execute a task.
///
/// # Fields
/// - `id` - The identifier of the task.
/// - `channel` - The channel of the scheduler handle. The channel is used to send instructions to the scheduler.
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::scheduler::SchedulerHandle;
/// use chronographer::scheduler::SchedulerHandleInstructions;
///
/// let scheduler = Scheduler::<Box<dyn std::error::Error + Send + Sync>>::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// let handle = SchedulerHandle {
///     id: Arc::new(id),
///     channel: scheduler.instruction_channel.lock().await.unwrap().clone(), // get the instruction channel from the scheduler
/// };
/// handle.instruct(SchedulerHandleInstructions::Reschedule).await;
/// ```
pub(crate) struct SchedulerHandle {
    pub(crate) id: Arc<dyn Any + Send + Sync>,
    pub(crate) channel: tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
}

impl SchedulerHandle {
    /// The `instruct` method for the `SchedulerHandle` type.
    ///
    /// # Usage
    /// Use the `instruct` method to instruct the scheduler to reschedule, halt, block, or execute a task. The instruction is sent to the scheduler via the channel.
    ///
    /// # Parameters
    /// - `instruction` - The instruction to send to the scheduler. For example: [`SchedulerHandleInstructions::Reschedule`](crate::scheduler::engine::default::SchedulerHandleInstructions::Reschedule).
    ///
    /// # Returns
    /// - `()` - The result of the instruction.
    pub(crate) async fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel
            .send((self.id.clone(), instruction))
            .await
            .expect("Cannot instruct");
    }
}

impl TaskHook<()> for SchedulerHandle {}

pub(crate) async fn append_scheduler_handler<C: SchedulerConfig>(
    task: &ErasedTask<C::TaskError>,
    id: C::TaskIdentifier,
    channel: &tokio::sync::mpsc::Sender<SchedulerHandlePayload>,
) {
    let handle = SchedulerHandle {
        id: Arc::new(id),
        channel: channel.clone(),
    };

    task.attach_hook::<()>(Arc::new(handle)).await;
}

impl<C: SchedulerConfig> Scheduler<C> {
    pub fn builder() -> SchedulerInitConfigBuilder<C> {
        SchedulerInitConfig::builder()
    }

    pub async fn start(&self) {
        let process_lock = self.process.lock().await;
        if process_lock.is_some() {
            return;
        }
        drop(process_lock);

        let engine_clone = self.engine.clone();
        let clock_clone = self.clock.clone();
        let store_clone = self.store.clone();
        let dispatcher_clone = self.dispatcher.clone();

        let channel = join!(
            self.clock.init(),
            self.store.init(),
            self.dispatcher.init(),
            self.engine.init(),
            self.engine
                .create_instruction_channel(&clock_clone, &store_clone, &dispatcher_clone)
        )
        .4;

        for (id, task) in self.store.iter() {
            append_scheduler_handler::<C>(&task, id, &channel).await;
        }

        *self.instruction_channel.lock().await = Some(channel);

        *self.process.lock().await = Some(tokio::spawn(async move {
            engine_clone
                .main(clock_clone, store_clone, dispatcher_clone)
                .await;
        }))
    }

    pub async fn abort(&self) {
        let process = self.process.lock().await.take();
        if let Some(p) = process {
            p.abort();
        }
    }

    pub async fn clear(&self) {
        self.store.clear().await;
    }

    pub async fn schedule(
        &self,
        task: &Task<impl TaskFrame<Error = C::TaskError>, impl TaskTrigger>,
    ) -> Result<C::TaskIdentifier, Box<dyn Error + Send + Sync>> {
        let erased = task.as_erased();
        let id = C::TaskIdentifier::generate();
        if let Some(channel) = &*self.instruction_channel.lock().await {
            append_scheduler_handler::<C>(&erased, id.clone(), channel).await;
        }
        self.store.store(&self.clock, id, erased).await
    }

    pub async fn cancel(&self, idx: &C::TaskIdentifier) {
        self.store.remove(idx).await;
    }

    pub fn exists(&self, idx: &C::TaskIdentifier) -> bool {
        self.store.exists(idx)
    }

    pub async fn has_started(&self) -> bool {
        self.process.lock().await.is_some()
    }
}
