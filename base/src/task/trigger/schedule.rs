//! This module contains various implementations of scheduling primitives via [`TaskTrigger`](crate::task::TaskTrigger).
//!
//! When it comes to most use cases, the built-in scheduling primitives are most used. However, depending
//! on your needs, you may implement the [`TaskTrigger`](crate::task::TaskTrigger) trait for a custom schedule.
//!
//! # Exports
//! - [`TaskSchedule`] - An alias trait of [`TaskTrigger`](crate::task::TaskTrigger) for mathematical & immediate computations.
//! - [`TaskScheduleImmediate`] - A primitive which schedules to execute immediately.
//! - [`TaskScheduleInterval`] - A primitive which schedules per-interval basis.
//! - [`TaskScheduleCron`] - A primitive which schedules based on a CRON expression.
//! - [`CronField`] - A field used internally for [`TaskScheduleCron`]
//! - [`TaskScheduleCalendar`] - A primitive which schedules via a human-readable calendar object.
//! - [`TaskCalendarField`] - A field of [`TaskScheduleCalendar`] which allows complex scheduling.
//!
//! # Example(s)
//! TODO: Expand upon the Example(s) once you are finished with documenting the other primitives
//!
//! Implementing your own custom schedule? Best refer to [`TaskTrigger`](crate::task::TaskTrigger) documentation
//!
//! # See Also
//! - [`TaskScheduleImmediate`] - A primitive which schedules to execute immediately.
//! - [`TaskScheduleInterval`] - A primitive which schedules per-interval basis.
//! - [`TaskScheduleCron`] - A primitive which schedules based on a CRON expression.
//! - [`CronField`] - A field used internally for [`TaskScheduleCron`]
//! - [`TaskScheduleCalendar`] - A primitive which schedules via a human-readable calendar object.
//! - [`TaskCalendarField`] - A field of [`TaskScheduleCalendar`] which allows complex scheduling.
//! - [`TaskTrigger`](crate::task::TaskTrigger) - The trait for managing scheduling / trigger logic.

mod calendar; // skipcq: RS-D1001
mod cron; // skipcq: RS-D1001
mod immediate;
mod interval; // skipcq: RS-D1001

pub mod cron_lexer; // skipcq: RS-D1001
pub mod cron_parser; // skipcq: RS-D1001

pub use calendar::*;
pub use cron::*;
pub use immediate::*;
pub use interval::*;