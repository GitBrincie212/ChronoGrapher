pub mod errors; // skipcq: RS-D1001

pub mod scheduler; // skipcq: RS-D1001

pub mod task; // skipcq: RS-D1001

pub mod utils; // skipcq: RS-D1001

pub mod persistence; // skipcq: RS-D1001

pub mod prelude {
    // Core
    pub use crate::task::{Task, TaskContext, TaskError};
    pub use crate::scheduler::CHRONOGRAPHER_SCHEDULER;

    // Common frames
    pub use crate::task::delayframe::DelayTaskFrame;
    pub use crate::task::timeoutframe::TimeoutTaskFrame;
    pub use crate::task::retryframe::RetriableTaskFrame;
    pub use crate::task::fallbackframe::FallbackTaskFrame;
    pub use crate::task::sequentialframe::SequentialTaskFrame;
    pub use crate::task::parallelframe::ParallelTaskFrame;
    pub use crate::task::dependencyframe::DependencyTaskFrame;
    pub use crate::task::dynamicframe::DynamicTaskFrame;

    // Scheduling
    pub use crate::task::schedule::TaskScheduleInterval;
    pub use crate::task::schedule::cron::TaskScheduleCron;
    pub use crate::task::schedule::calendar::TaskScheduleCalendar;
    pub use crate::task::scheduling_strats::{
        SequentialSchedulingPolicy,
        ConcurrentSchedulingPolicy,
        CancelPreviousSchedulingPolicy,
        CancelCurrentSchedulingPolicy,
    };

    // TaskHooks / TaskHookEvents
    pub use crate::task::hooks::{
        TaskHook,
        NonObserverTaskHook,
        events::*
    };

    // Utils / Misc
    pub use crate::task::TaskFrameBuilder;
    pub use crate::task::dependency::*;
    pub use crate::task::retryframe::{
        RetryBackoffStrategy,
        ExponentialBackoffStrategy
    };
} // skipcq: RS-D1001