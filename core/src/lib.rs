pub mod errors; // skipcq: RS-D1001

pub mod scheduler; // skipcq: RS-D1001

pub mod task; // skipcq: RS-D1001

pub mod utils; // skipcq: RS-D1001

pub mod prelude {
    // Core
    pub use crate::task::{Task, RestrictTaskFrameContext, DynArcError};
    pub use crate::errors::TaskError;

    // Common frames
    pub use crate::task::delayframe::DelayTaskFrame;
    pub use crate::task::dependencyframe::DependencyTaskFrame;
    pub use crate::task::dynamicframe::DynamicTaskFrame;
    pub use crate::task::fallbackframe::FallbackTaskFrame;
    pub use crate::task::collectionframe::CollectionTaskFrame;
    pub use crate::task::retryframe::RetriableTaskFrame;
    pub use crate::task::timeoutframe::TimeoutTaskFrame;

    // Scheduling
    pub use crate::task::trigger::TaskScheduleInterval;
    pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
    pub use crate::task::trigger::schedule::cron::TaskScheduleCron;

    // Schedulers
    pub use crate::scheduler::DefaultSchedulerConfig;
    pub use crate::scheduler::Scheduler;
    pub use crate::scheduler::DefaultScheduler;
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
