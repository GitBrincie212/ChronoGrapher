#[allow(unused_imports)]
use crate::task::Task;

#[allow(unused_imports)]
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;

/// [`TaskPriority`] dictates the importance of a [`Task`], the more important a task is,
/// the more ChronoGrapher ensures to execute the task at a specific time without any latency,
/// no matter what. Priorities are mostly used and handled in [`SchedulerTaskDispatcher`]
///
/// # Variants
/// - [`TaskPriority::LOW`] It is the lowest level of priority and represents a non-important background task,
/// time drifts are bound to happen under heavy workflow
///
/// - [`TaskPriority::MODERATE`] It is a semi non-important background task, slight time drifts may
/// happen under heavier workflow. Also, the default variant for most tasks
///
/// - [`TaskPriority::HIGH`] It is slightly important task, time drifts are rarer but when they happen
/// they are at such small barely noticeable scale under very heavy workflow
///
/// - [`TaskPriority::IMPORTANT`] It is an important task, time drifts are improbable but still possible,
/// again they are at such small barely noticeable scale under extreme heavy workflow. ChronoGrapher
/// ensures that this type of priority is executed at almost the exact time
///
/// - [`TaskPriority::CRITICAL`] It is the highest level of priority, and represents a critical task
/// that has to be executed at this time specifically. No time drifts are allowed, exact laser focus
/// execution
///
/// # Trait Implementation(s)
/// The enum is cloneable and copiable via [`Clone`] and [`Copy`] traits respectively, it also
/// implements the [`Debug`] trait, as well as equality and ord via [`PartialEq`], [`PartialOrd`],
/// [`Ord`] and [`Eq`]. In addition to the [`Default`] trait with default value [`TaskPriority::MODERATE`]
///
/// # See Also
/// - [`Task`]
/// - [`SchedulerTaskDispatcher`]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// It is the highest level of priority, and represents a critical task
    /// that has to be executed at this time specifically. No time drifts are allowed, exact laser focus
    /// execution
    CRITICAL,

    /// It is an important task, time drifts are improbable but still possible,
    // again, they are at such small barely noticeable scale under extreme heavy workflow. ChronoGrapher
    // ensures that this type of priority is executed at almost the exact time
    IMPORTANT,

    /// It is a slightly important task, time drifts are rarer but when they happen
    /// they are at such small barely noticeable scale under very heavy workflow
    HIGH,

    /// It is a semi non-important background task, slight time drifts may
    /// happen under heavier workflow. Also, the default variant for most tasks
    #[default]
    MODERATE,

    /// It is the lowest level of priority and represents a non-important background task,
    /// time drifts are bound to happen under heavy workflow
    LOW,
}
