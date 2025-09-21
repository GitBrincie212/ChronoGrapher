/// [`TaskPriority`] dictates the importance of a task, the more important a task is,
/// the more ChronoGrapher ensures to execute the task at a specific time without any latency,
/// no matter what, the lower level of the spectrum is [`TaskPriority::LOW`] while the highest it
/// can be is [`TaskPriority::CRITICAL`] where no single time drift is allowed
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    CRITICAL,
    IMPORTANT,
    HIGH,

    #[default]
    MODERATE,

    LOW,
}
