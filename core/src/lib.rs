pub mod errors; // skipcq: RS-D1001

pub mod scheduler; // skipcq: RS-D1001

pub mod task; // skipcq: RS-D1001

pub mod utils; // skipcq: RS-D1001

pub mod prelude {
    // Core
    pub use crate::scheduler::CHRONOGRAPHER_SCHEDULER;
    pub use crate::task::{Task, TaskContext, DynArcError};

    // Common frames
    pub use crate::task::delayframe::DelayTaskFrame;
    pub use crate::task::dependencyframe::DependencyTaskFrame;
    pub use crate::task::dynamicframe::DynamicTaskFrame;
    pub use crate::task::fallbackframe::FallbackTaskFrame;
    pub use crate::task::parallelframe::ParallelTaskFrame;
    pub use crate::task::retryframe::RetriableTaskFrame;
    pub use crate::task::sequentialframe::SequentialTaskFrame;
    pub use crate::task::timeoutframe::TimeoutTaskFrame;

    // Scheduling
    pub use crate::task::scheduling_strats::{
        CancelCurrentSchedulingPolicy, CancelPreviousSchedulingPolicy, ConcurrentSchedulingPolicy,
        SequentialSchedulingPolicy,
    };
    pub use crate::task::trigger::TaskScheduleInterval;
    pub use crate::task::trigger::schedule::calendar::TaskScheduleCalendar;
    pub use crate::task::trigger::schedule::cron::TaskScheduleCron;

    // TaskHooks / TaskHookEvents
    pub use crate::task::hooks::{NonObserverTaskHook, TaskHook, events::*};

    // Utils / Misc
    pub use crate::task::TaskFrameBuilder;
    pub use crate::task::dependency::*;
    pub use crate::task::retryframe::{ExponentialBackoffStrategy, RetryBackoffStrategy};
} // skipcq: RS-D1001
