/// Error types and the shared [`TaskError`] trait used across tasks, frames, triggers, and the scheduler.
///
/// # [`TaskError`](crate::errors::TaskError) trait
/// [`TaskError`] is the common bound for all task-related errors: `Debug + Display + Send + Sync + 'static`,
/// with [`TaskError::as_any`](crate::errors::TaskError::as_any) for downcasting. Concrete types implement it via a blanket impl.
///
/// # Error catagories:
///
/// - **Frame errors** - Wrappers for frame execution failures:
///   - [`ConditionalTaskFrameError`] - primary/secondary branch or condition returned false.
///   - [`TimeoutTaskFrameError`] - inner error or timeout exceeded.
///   - [`DependencyTaskFrameError`] - inner failure or dependencies invalidated.
/// - **Cron errors** - Parsing and lexing of cron expressions:
///   - [`CronError`], [`CronErrorTypes`], [`CronExpressionParserErrors`], [`CronExpressionLexerErrors`].
/// - **Core / scheduler** - General runtime and scheduler issues:
///   - [`StandardCoreErrorsCG`] - index out of bounds, unresolved dependencies, invalid cron, unsupported instructions, etc.
///
/// Use the types that match the API you call (e.g. timeout frame returns `TimeoutTaskFrameError`).
/// [`StandardCoreErrorsCG`] is the common enum for core-level failures.
///
/// # Error Types:
/// - [`ConditionalTaskFrameError`](crate::errors::ConditionalTaskFrameError)
///   - Represents errors that occur within a conditional task frame.
///   - Contains both primary and secondary task frame errors.
/// - [`TimeoutTaskFrameError`](crate::errors::TimeoutTaskFrameError)
///   - Represents errors that occur within a timeout task frame.
///   - Contains the inner task frame error and the timeout duration.
/// - [`DependencyTaskFrameError`](crate::errors::DependencyTaskFrameError)
///   - Represents errors that occur within a dependency task frame.
///   - Contains the inner task frame error and the dependencies invalidated error.
/// - [`CronError`](crate::errors::CronError)
///   - Represents errors that occur within a cron task frame.
///   - Contains the `field position`, `position`, and `error type`.
///
/// # Usage:
///
/// Import the [`TaskError`] trait when handling task/frame/scheduler results, and use the concrete
/// error types that match the API you call (e.g. a timeout frame returns [`TimeoutTaskFrameError`]).
/// Use [`TaskError::as_any`] for downcasting when you need a concrete type. For core/scheduler
/// failures, match on or propagate [`StandardCoreErrorsCG`].
///
/// # Example:
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::errors::*;
///
/// let result = task_frame.execute(&TaskFrameContext::new()).await;
/// if let Err(e) = result {
///     if let Ok(timeout_error) = e.as_any().downcast_ref::<TimeoutTaskFrameError>() {
///         println!("TimeoutTaskFrameError: {}", timeout_error);
///     } else {
///         println!("Other error: {}", e);
///     }
/// }
/// ```
pub mod errors; // skipcq: RS-D1001

/// The `scheduler` module provides the runtime that runs [`Task`](crate::task::Task)s according to their
/// [`TaskTrigger`](crate::task::TaskTrigger)s, plus configuration and default implementations.
///
/// This module defines:
/// - **[`Scheduler`](crate::scheduler::Scheduler)** - The main type that owns a clock, task store, dispatcher, and engine;
///   use [`Scheduler::builder`](crate::scheduler::Scheduler::builder) or `Default::default()` to construct.
/// - **[`SchedulerConfig`](crate::scheduler::SchedulerConfig)** - Trait that configures identifier, error type, clock, store, dispatcher, and engine.
/// - **[`DefaultScheduler`](crate::scheduler::DefaultScheduler)** / **[`DefaultSchedulerConfig`](crate::scheduler::DefaultSchedulerConfig)** - Default implementation;
///   generic over the error type `E: [`TaskError`](crate::errors::TaskError)`.
/// - **Feature-gated schedulers**:
///   - [`DefaultAnyhowScheduler`] (`anyhow` feature)
///   - [`DefaultEyreScheduler`] (`eyre` feature)
///
/// Main operations: [`start`](crate::scheduler::Scheduler::start), [`schedule`](crate::scheduler::Scheduler::schedule),
/// [`cancel`](crate::scheduler::Scheduler::cancel), [`clear`](crate::scheduler::Scheduler::clear), [`abort`](crate::scheduler::Scheduler::abort).
///
/// # Features:
///
/// - `anyhow` - [`DefaultAnyhowScheduler`]
/// - `eyre` - [`DefaultEyreScheduler`]
///
/// # Usage:
/// Build a scheduler with default config, start it, then schedule tasks. Use the [`prelude`] or
/// [`DefaultScheduler`] and [`DefaultSchedulerConfig`] for the default implementation.
///
/// # Example:
/// ```rust
/// use chronographer::prelude::*;
///
/// let scheduler = DefaultScheduler::<Box<dyn std::error::Error + Send + Sync>>::default();
/// scheduler.start().await;
/// let id = scheduler.schedule(&my_task).await?;
/// ```
pub mod scheduler; // skipcq: RS-D1001

/// The `task` module provides the core [`Task`](crate::task::Task) abstraction and related context types.
///
/// # Purpose
///
/// This module defines the main types used to represent runnable work and when it runs:
///
/// - **[`Task`]** — The main type that pairs a [`TaskFrame`] with a [`TaskTrigger`];
///   use [`Task::new`](crate::task::Task::new) or `Default::default()` to construct, and [`Task::as_erased`](crate::task::Task::as_erased) to get an [`ErasedTask`](crate::task::ErasedTask) for the scheduler.
/// - **[`ErasedTask`]** — Type-erased task used by the scheduler and anywhere you need a single task type regardless of frame/trigger; one of the central types in the crate. Obtained via [`Task::as_erased`].
/// - **[`RestrictTaskFrameContext`]** — Context passed into task frames during execution.
/// - **Hooks** — Observers that react to task and frame events (e.g. [`OnTaskStart`], [`OnTaskEnd`], retry, timeout). Implement [`TaskHook`]<`E`> for an event type `E`: [TaskHookEvent] and attach via [`Task::attach_hook`]. [`TaskHookEvent`] defines the event and payload; [`TaskHookContext`] is passed to handlers; [`TaskHookContainer`] holds hooks on each task.
///
/// The module also re-exports frame and trigger types from its submodules (e.g. [`TaskFrame`], [`TaskTrigger`], [`TaskFrameBuilder`], and the concrete frame and schedule types).
///
/// # Usage
///
/// Build a [`Task`] with [`Task::new`](trigger, frame), call [`as_erased`] to get an [`ErasedTask`], then pass that to [`Scheduler::schedule`] or run it with [`ErasedTask::run`].
pub mod task; // skipcq: RS-D1001

/// The [`utils`] module provides utility types and functions for the core library.
/// It mainly defines the [`TaskIdentifier`](crate::utils::TaskIdentifier) trait.
///
/// # Usage
///
/// Use the [`TaskIdentifier`](crate::utils::TaskIdentifier) trait to define a unique identifier for a task.
/// Use the [`DefaultTaskID`](crate::utils::DefaultTaskID) type as the default implementation of the [`TaskIdentifier`](crate::utils::TaskIdentifier) trait.
///
/// ```rust
/// use uuid::Uuid;
/// use chronographer::prelude::*;
/// use chronographer::utils::*;
///
/// struct TaskId(Uuid);
/// impl TaskIdentifier for TaskId {
///     fn generate() -> Self {
///         TaskId(Uuid::new_v4())
///     }
/// }
///
/// let task_id = TaskId::generate();
/// assert_ne!(task_id, TaskId::generate()); // Unequal, as they are unique
/// ```
///
/// You can also use the [`Uuid`](uuid::Uuid) type as the implementation of the [`TaskIdentifier`](crate::utils::TaskIdentifier) trait.
/// In addition, you can use macros to define your own event types and event groups.
///
/// # Example
/// ```rust
/// use chronographer::prelude::*;
/// use chronographer::utils::*;
///
/// define_event!(MyEvent, MyEventPayload);
/// define_event_group!(MyEventGroup, MyEvent, MyEvent2);
/// ```
pub mod utils; // skipcq: RS-D1001

/// The `prelude` module re-exports the most commonly used types, traits, frames,
/// schedulers, hooks, and utilities of the core library.
///
/// This is useful for quickly getting started with the core library and for those who want to use the core library
/// without having to import each module individually. Reduces boilerplate code for common use cases.
///
/// # Purpose:
/// Importing this module brings into scope:
///
/// - The core [`Task`](crate::task::Task) abstraction and related context types.
/// - Standard task frame wrappers (retry, timeout, fallback, dependency, etc.).
/// - Scheduling primitives ([`TaskScheduleInterval`](crate::task::trigger::TaskScheduleInterval), [`TaskScheduleCalendar`](crate::task::trigger::schedule::calendar::TaskScheduleCalendar), [`TaskScheduleCron`](crate::task::trigger::schedule::cron::TaskScheduleCron)).
/// - Default scheduler implementations and configuration types ([`DefaultScheduler`](crate::scheduler::DefaultScheduler), [`DefaultSchedulerConfig`](crate::scheduler::DefaultSchedulerConfig)).
/// - Task hooks and hook event types ([`TaskHook`](crate::task::hooks::TaskHook), [`TaskHookEvent`](crate::task::hooks::TaskHookEvent)).
/// - Common retry strategies and dependency utilities ([`ExponentialBackoffStrategy`](crate::task::retryframe::ExponentialBackoffStrategy), [`RetryBackoffStrategy`](crate::task::retryframe::RetryBackoffStrategy)).
/// - The [`TaskFrameBuilder`](crate::task::TaskFrameBuilder) for fluent workflow composition.
///
/// This module does not define new types - it only re-exports selected public API
/// items to simplify ergonomics for library users.
///
/// # Usage:
/// ```rust
/// use chronographer::prelude::*;
/// ```
///
/// # Re-Export Groups
///
/// ## Core
/// - \[`TaskError`\]
/// - \[`Task`\]
/// - \[`RestrictTaskFrameContext`\]
///
/// ## Common Task Frames
/// - \[`CollectionTaskFrame`\]
/// - \[`DelayTaskFrame`\]
/// - \[`DependencyTaskFrame`\]
/// - \[`DynamicTaskFrame`\]
/// - \[`FallbackTaskFrame`\]
/// - \[`RetriableTaskFrame`\]
/// - \[`TimeoutTaskFrame`\]
///
/// ## Scheduling
/// - \[`TaskScheduleInterval`\]
/// - \[`TaskScheduleCalendar`\]
/// - \[`TaskScheduleCron`\]
///
/// ## Schedulers
/// - \[`Scheduler`\]
/// - \[`SchedulerConfig`\]
/// - \[`DefaultScheduler`\]
/// - \[`DefaultSchedulerConfig`\]
///
/// Feature-gated schedulers:
/// - `anyhow` → \[`DefaultAnyhowScheduler`\]
/// - `eyre` → \[`DefaultEyreScheduler`\]
///
/// ## Hooks
/// - \[`TaskHook`\]
/// - \[`NonObserverTaskHook`\]
/// - All hook event types under \[`events`\]
///
/// ## Utilities
/// - \[`TaskFrameBuilder`\]
/// - Dependency utilities under \[`dependency`\]
/// - \[`RetryBackoffStrategy`\]
/// - \[`ExponentialBackoffStrategy`\]
///
/// # Design Notes
///
/// The prelude follows the conventional Rust pattern of exposing a curated,
/// stability-oriented surface of the public API. Not all internal modules are
/// re-exported here - only those considered commonly required for application-level
/// usage.
///
/// Advanced or low-level functionality should be imported directly from its
/// originating module.
pub mod prelude {
    // Core
    pub use crate::errors::TaskError;
    pub use crate::task::{RestrictTaskFrameContext, Task};

    // Common frames
    pub use crate::task::collectionframe::CollectionTaskFrame;
    pub use crate::task::collectionframe::GroupedTaskFramesQuitOnFailure;
    pub use crate::task::collectionframe::GroupedTaskFramesQuitOnSuccess;
    pub use crate::task::collectionframe::GroupedTaskFramesSilent;
    pub use crate::task::collectionframe::ParallelExecStrategy;
    pub use crate::task::collectionframe::SelectFrameAccessor;
    pub use crate::task::collectionframe::SelectionExecStrategy;
    pub use crate::task::collectionframe::SequentialExecStrategy;
    pub use crate::task::delayframe::DelayTaskFrame;
    pub use crate::task::dependencyframe::DependencyTaskFrame;
    pub use crate::task::dynamicframe::DynamicTaskFrame;
    pub use crate::task::fallbackframe::FallbackTaskFrame;
    pub use crate::task::retryframe::RetriableTaskFrame;
    pub use crate::task::timeoutframe::TimeoutTaskFrame;

    // Scheduling
    pub use crate::task::trigger::TaskScheduleInterval;
    pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
    pub use crate::task::trigger::schedule::cron::TaskScheduleCron;

    // Schedulers
    pub use crate::scheduler::DefaultScheduler;
    pub use crate::scheduler::DefaultSchedulerConfig;
    pub use crate::scheduler::Scheduler;
    pub use crate::scheduler::SchedulerConfig;

    #[cfg(feature = "anyhow")]
    pub use crate::scheduler::DefaultAnyhowScheduler;

    #[cfg(feature = "eyre")]
    pub use crate::scheduler::DefaultEyreScheduler;

    // TaskHooks / TaskHookEvents
    pub use crate::task::hooks::{NonObserverTaskHook, TaskHook, events::*};

    // Utils / Misc
    pub use crate::task::TaskFrameBuilder;
    pub use crate::task::dependency::*;
    pub use crate::task::retryframe::{ExponentialBackoffStrategy, RetryBackoffStrategy};
} // skipcq: RS-D1001
