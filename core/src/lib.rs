pub use chronographer_base::*;

#[cfg(feature = "macros")]
pub use chronographer_macros::*;

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! dynamic_taskframe {
    ($block: block) => {{
        DynamicTaskFrame::new(|taskframe_ctx| async {
            $block;
            Ok(())
        })
    }};
}

#[cfg(feature = "macros")]
macro_rules! import_macros {
    () => {
        pub use chronographer_macros::every;
        pub use chronographer_macros::task;
        pub use chronographer_macros::taskframe;
    };
}


pub mod prelude {
    // Macros
    #[cfg(feature = "macros")]
    import_macros!();

    // Core
    pub use crate::errors::TaskError;
    pub use crate::task::{TaskFrameContext, RestrictTaskFrameContext, Task};

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

    // Scheduling / Triggering
    pub use crate::task::schedule::TaskScheduleInterval;
    pub use crate::task::schedule::TaskScheduleCalendar;
    pub use crate::task::schedule::TaskCalendarField;
    pub use crate::task::schedule::TaskScheduleCron;
    pub use crate::task::schedule::TaskSchedule;

    // Schedulers
    pub use crate::scheduler::DefaultLiveScheduler;
    pub use crate::scheduler::LiveScheduler;
    pub use crate::scheduler::DefaultSchedulerConfig;
    pub use crate::scheduler::Scheduler;
    pub use crate::scheduler::SchedulerConfig;
    pub use crate::scheduler::FailoverPolicy;

    #[cfg(feature = "anyhow")]
    pub use crate::scheduler::DefaultLiveAnyhowScheduler;

    #[cfg(feature = "eyre")]
    pub use crate::scheduler::DefaultLiveEyreScheduler;

    // TaskHooks / TaskHookEvents
    pub use crate::task::hooks::{NonObserverTaskHook, TaskHook, events::*};

    // Utils / Misc
    pub use crate::task::TaskFrameBuilder;
    pub use crate::task::dependency::*;
    pub use crate::task::retryframe::{
        ExponentialBackoffStrategy, LinearBackoffStrategy, ConstantBackoffStrategy,
        JitterBackoffStrategy, RetryBackoffStrategy
    };
} // skipcq: RS-D1001
