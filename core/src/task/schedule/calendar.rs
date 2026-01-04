use crate::task::{TaskError, TaskSchedule};
use chrono::{DateTime, Datelike, Local, LocalResult, NaiveDate, TimeZone, Timelike};
use std::fmt::Debug;
use std::ops::{Bound, Deref, RangeBounds};

/// [`TaskCalendarField`] is a trait that defines a field on the schedule,
/// by itself it just holds data and how this data is scheduled, it is useful for
/// [`TaskSchedule::Calendar`] only, all fields used in the calendar are zero-based
/// (they start from zero), fields have their own ranges defined, typically:
/// - **Year** can be any value (unrestricted)
/// - **Month** must be between 0 and 11 range
/// - **Day** must be between 0 and 30 range
/// - **Hour** must be between 0 and 23
/// - **Minute** must be between 0 and 59
/// - **Second** must be between 0 and 59
/// - **Millisecond** must be between 0 and 999
///
/// All ranges are <u>inclusive on both ends</u>, the scheduler auto-validates the field schedules and if they
/// are out of bounds, it panics with the corresponding error
///
/// # Required Method(s)
/// When implementing the [`TaskCalendarField`] trait, one has to supply an implementation for
/// the method [`TaskCalendarField::evaluate`] which hosts the logic for modifying the date field
/// to a corresponding value
pub trait TaskCalendarField: Send + Sync {
    /// This method is where the logic for modifying the date field lives in.
    ///
    /// # Argument(s)
    /// This method hosts 2 arguments, those being the ``date_field`` to modify as well as the
    /// ``date_field_type`` to indicate which date field the [`TaskCalendarField`] is modifying
    ///
    /// # See Also
    /// - [`TaskCalendarField`]
    fn evaluate(&self, date_fields: &mut [u32], idx: usize);
}

impl<T> TaskCalendarField for T
where
    T: Deref + Send + Sync,
    T::Target: TaskCalendarField,
{
    fn evaluate(&self, date_field: &mut [u32], idx: usize) {
        self.deref().evaluate(date_field, idx)
    }
}

/// [`TaskCalendarFieldIdentity`] is an implementation of the trait [`TaskCalendarField`],
/// where it keeps the same date field intact without any modification
///
/// # Constructor(s)
/// When constructing [`TaskCalendarFieldIdentity`], one can do it via rust's struct initialization
/// or from [`TaskCalendarFieldIdentity::default`] via [`Default`]
///
/// # Trait Implementation(s)
/// Obviously [`TaskCalendarFieldIdentity`] implements the [`TaskCalendarField`] trait, but also
/// implements the [`Debug`] trait, the [`Default`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`TaskCalendarField`]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldIdentity;

impl TaskCalendarField for TaskCalendarFieldIdentity {
    fn evaluate(&self, _date_field: &mut [u32], _idx: usize) {}
}

/// [`TaskCalendarFieldExact`] is an implementation of the trait [`TaskCalendarField`],
/// where it modifies the date field to have an exact value **always**
///
/// # Constructor(s)
/// When constructing [`TaskCalendarFieldExact`], one can do it via [`TaskCalendarFieldExact::new`]
/// where they can supply a ``value`` to always modify to
///
/// # Trait Implementation(s)
/// Obviously [`TaskCalendarFieldExact`] implements the [`TaskCalendarField`] trait, but also
/// implements the [`Debug`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`TaskCalendarField`]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldExact(u32);

impl TaskCalendarFieldExact {
    /// Creates / Constructs a new [`TaskCalendarFieldExact`] instance
    ///
    /// # Argument(s)
    /// This method requires only one argument, this being ``value`` which is a ``u32``
    /// number, it is the value that will always modify the field to
    ///
    /// # Returns
    /// The newly constructed [`TaskCalendarFieldExact`] instance with the value
    /// to modify the target date field to being ``value``
    ///
    /// # See Also
    /// - [`TaskCalendarFieldExact`]
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

impl TaskCalendarField for TaskCalendarFieldExact {
    fn evaluate(&self, date_field: &mut [u32], idx: usize) {
        if self.0 < date_field[idx] {
            date_field[(idx + 1).min(6)] += 1;
        }
        date_field[idx] = self.0
    }
}

/// [`TaskCalendarFieldExact`] is an implementation of the trait [`TaskCalendarField`],
/// where it modifies the date field to have an exact value **always**
///
/// # Constructor(s)
/// When constructing [`TaskCalendarFieldExact`], one can do it via [`TaskCalendarFieldExact::new`]
/// where they can supply a ``value`` to always modify to
///
/// # Trait Implementation(s)
/// Obviously [`TaskCalendarFieldExact`] implements the [`TaskCalendarField`] trait, but also
/// implements the [`Debug`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`TaskCalendarField`]
#[derive(Debug, Clone)]
pub struct TaskCalendarFieldRange<T: TaskCalendarField>(u32, Option<u32>, T);

impl<T: TaskCalendarField> TaskCalendarFieldRange<T> {
    /// Creates / Constructs a new [`TaskCalendarFieldExact`] instance
    ///
    /// # Argument(s)
    /// This method requires only one argument, this being ``value`` which is a ``u32``
    /// number, it is the value that will always modify the field to
    ///
    /// # Returns
    /// The newly constructed [`TaskCalendarFieldExact`] instance with the value
    /// to modify the target date field to being ``value``
    ///
    /// # See Also
    /// - [`TaskCalendarFieldExact`]
    pub fn new(range: impl RangeBounds<u32>, field: T) -> Option<Self> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => Some(*end),
            Bound::Excluded(end) => Some(end - 1),
            Bound::Unbounded => None,
        };
        if end <= Some(start) {
            return None;
        }
        Some(Self(start, end, field))
    }
}

impl<T: TaskCalendarField> TaskCalendarField for TaskCalendarFieldRange<T> {
    fn evaluate(&self, date_field: &mut [u32], idx: usize) {
        let end_bound = match idx {
            6 => u32::MAX,
            5 => 11,
            4 => 31,
            3 => 23,
            2 => 59,
            1 => 59,
            0 => 999,
            _ => unreachable!(),
        };
        let end = self.1.unwrap_or(end_bound).min(end_bound);
        let start = self.0;
        let range_size = end - start + 1;
        let prev_date_field: u32 = date_field[idx];
        self.2.evaluate(date_field, idx);
        let diff = date_field[idx] - prev_date_field;
        if diff == 0 {
            return;
        }
        if date_field[idx] > end {
            let cycles_above = (date_field[idx] - end - 1) / range_size + 1;
            date_field[(idx + 1).min(6)] += cycles_above;
        }
        date_field[idx] = date_field[idx] % range_size + start;
    }
}

/// [`TaskCalendarFieldInterval`] is an implementation of the trait [`TaskCalendarField`],
/// where it modifies the date field by adding an interval
///
/// # Constructor(s)
/// When constructing [`TaskCalendarFieldInterval`], one can do it via [`TaskCalendarFieldExact::new`]
/// where they can supply a ``interval`` to always add onto
///
/// # Trait Implementation(s)
/// Obviously [`TaskCalendarFieldInterval`] implements the [`TaskCalendarField`] trait, but also
/// implements the [`Debug`] trait, the [`Clone`] trait and the [`Copy`] trait
///
/// # See Also
/// - [`TaskCalendarField`]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldInterval(u32);

impl TaskCalendarFieldInterval {
    /// Creates / Constructs a new [`TaskCalendarFieldInterval`] instance
    ///
    /// # Argument(s)
    /// This method requires only one argument, this being ``interval`` which is a ``u32``
    /// number, it is the interval that will be added onto the date_field
    ///
    /// # Returns
    /// The newly constructed [`TaskCalendarFieldInterval`] instance with the interval
    /// being ``interval``
    ///
    /// # See Also
    /// - [`TaskCalendarFieldInterval`]
    pub fn new(interval: u32) -> Self {
        Self(interval)
    }
}

impl TaskCalendarField for TaskCalendarFieldInterval {
    fn evaluate(&self, date_field: &mut [u32], idx: usize) {
        date_field[idx] = date_field[idx].saturating_add(self.0);
    }
}

/// An internal type used to denote this is a builder
pub struct TaskCalendarBuilderField<T>(pub T);

pub type TaskScheduleCalendarBuilder<
    Year = TaskCalendarFieldIdentity,
    Month = TaskCalendarFieldIdentity,
    Day = TaskCalendarFieldIdentity,
    Hour = TaskCalendarFieldIdentity,
    Minute = TaskCalendarFieldIdentity,
    Second = TaskCalendarFieldIdentity,
    Millisecond = TaskCalendarFieldIdentity,
> = TaskScheduleCalendar<
    TaskCalendarBuilderField<Year>,
    TaskCalendarBuilderField<Month>,
    TaskCalendarBuilderField<Day>,
    TaskCalendarBuilderField<Hour>,
    TaskCalendarBuilderField<Minute>,
    TaskCalendarBuilderField<Second>,
    TaskCalendarBuilderField<Millisecond>,
>;

/// [`TaskScheduleCalendar`] is an implementation of the [`TaskSchedule`] trait that allows defining
/// schedules with fine-grained control over individual calendar fields.
///
/// Each field can be configured independently to restrict when the schedule should match.
/// By default, all fields are set to [`TaskCalendarFieldIdentity`], which means the field
/// is set to the current time's field in [`TaskScheduleCalendar::next_after`]
///
/// If you want precise zero times, it is advised to use [`TaskCalendarFieldExact`] and set the
/// time field to zero
///
/// # Constructor(s)
/// When constructing a [`TaskScheduleCalendar`], the only way to achieve is via
/// [`TaskScheduleCalendar::builder`] which prompts you to a builder style pattern
/// for configuring your calendar
///
/// # Trait Implementation(s)
/// [`TaskScheduleCalendar`] not only implements the [`TaskSchedule`] trait but also
/// the [`Clone`] trait in addition
///
/// # Cloning Semantics
/// When cloning, this creates a shallow copy (due to some limitations), in most cases
/// this is fine. But when tracking state, it is advised to build a new [`TaskScheduleCalendar`]
/// from scratch
///
/// # Examples
/// ```ignore
/// // Example: A schedule that runs every day at 12:30:00.00
/// use chronographer_core::schedule::{TaskScheduleCalendar, TaskCalendarField};
///
/// let schedule = TaskScheduleCalendar::builder()
///     .hour(TaskCalendarField::Exactly(12))
///     .minute(TaskCalendarField::Exactly(30))
///     .second(TaskCalendarField::Exactly(0))
///     .build();
/// ```
///
/// # See Also
/// - [`TaskSchedule`]
/// - [`TaskCalendarField`]
pub struct TaskScheduleCalendar<Year, Month, Day, Hour, Minute, Second, Millisecond> {
    year: Year,
    month: Month,
    day: Day,
    hour: Hour,
    minute: Minute,
    second: Second,
    millisecond: Millisecond,
}

/*
    Some macro magic is used to reduce boilerplate and the tracking of all possible methods.
    Since no other field will be added (No microseconds, no decade... etc.), we can safely assume
    its static
*/

/*
   Check if "something" is equal to one of the corresponding fields and return the corresponding
   sequence of tokens (falsey for false, truthy for true)
*/
macro_rules! switch {
    (Year, Year, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Month, Month, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Day, Day, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Hour, Hour, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Minute, Minute, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Second, Second, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    (Millisecond, Millisecond, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($truthy)*};
    ($other: ident, Year, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Month, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Day, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Hour, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Minute, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Second, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
    ($other: ident, Millisecond, [$($falsey:tt)*], [$($truthy:tt)*]) => {$($falsey)*};
}

// Generalizes the calendar's builder method plus documentation, actively uses switch! for generalization
macro_rules! calendar_builder_method {
    ($(#[$($attrss:tt)*])* $name: ident, $generic: ident) => {
        impl<
            Year,
            Month,
            Day,
            Hour,
            Minute,
            Second,
            Millisecond,
        > TaskScheduleCalendarBuilder<
            Year,
            Month,
            Day,
            Hour,
            Minute,
            Second,
            Millisecond,
        > {
            $(#[$($attrss)*])*
            ///
            /// # Method Behavior
            /// This builder parameter method can be chained with same types, but not via different
            /// types as it is greatly limited from the generics themselves
            ///
            /// # Default Value
            /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
            /// date field as the supplied time
            ///
            /// # See Also
            /// - [`TaskCalendarFieldIdentity`]
            /// - [`TaskCalendarFieldExact`]
            /// - [`TaskCalendarFieldRange`]
            /// - [`TaskCalendarFieldInterval`]
            /// - [`TaskCalendarField`]
            pub fn $name<T>(self, $name: T) -> TaskScheduleCalendarBuilder<
                // Per generic, it checks if $name is equal to the corresponding field,
                // if yes then use T otherwise keep the field as is
                switch!($generic, Year, [Year], [T]),
                switch!($generic, Month, [Month], [T]),
                switch!($generic, Day, [Day], [T]),
                switch!($generic, Hour, [Hour], [T]),
                switch!($generic, Minute, [Minute], [T]),
                switch!($generic, Second, [Second], [T]),
                switch!($generic, Millisecond, [Millisecond], [T]),
            > {
                // Same logic, however, it uses values instead
                TaskScheduleCalendarBuilder {
                    year: switch!($generic, Year, [self.year], [TaskCalendarBuilderField($name)]),
                    month: switch!($generic, Month, [self.month], [TaskCalendarBuilderField($name)]),
                    day: switch!($generic, Day, [self.day], [TaskCalendarBuilderField($name)]),
                    hour: switch!($generic, Hour, [self.hour], [TaskCalendarBuilderField($name)]),
                    minute: switch!($generic, Minute, [self.minute], [TaskCalendarBuilderField($name)]),
                    second: switch!($generic, Second, [self.second], [TaskCalendarBuilderField($name)]),
                    millisecond: switch!($generic, Millisecond, [self.millisecond], [TaskCalendarBuilderField($name)])
                }
            }
        }
    };
}

impl TaskScheduleCalendarBuilder {
    pub fn builder() -> Self {
        Self {
            year: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            month: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            day: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            hour: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            minute: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            second: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
            millisecond: TaskCalendarBuilderField(TaskCalendarFieldIdentity),
        }
    }
}

impl<Year, Month, Day, Hour, Minute, Second, Millisecond>
    TaskScheduleCalendarBuilder<Year, Month, Day, Hour, Minute, Second, Millisecond>
{
    pub fn build(
        self,
    ) -> TaskScheduleCalendar<Year, Month, Day, Hour, Minute, Second, Millisecond> {
        TaskScheduleCalendar::<Year, Month, Day, Hour, Minute, Second, Millisecond> {
            year: self.year.0,
            month: self.month.0,
            day: self.day.0,
            hour: self.hour.0,
            minute: self.minute.0,
            second: self.second.0,
            millisecond: self.millisecond.0,
        }
    }
}

calendar_builder_method!(
    /// The year field, it is the only unrestricted and can be any value (non-negative)
    year, Year
);

calendar_builder_method!(
    /// The month field has a valid range of **0-11** (inclusive) where `0 = January`, `11 = December`
    month, Month
);

calendar_builder_method!(
    /// The day of the month field has most of the time a valid range of **0-30** (inclusive),
    /// however, this range may not always hold true, in-fact in special occasions. For example,
    /// when the month is set to 2 (February), it is 28 days (and sometimes 29 days on leap
    /// years)
    day, Day
);

calendar_builder_method!(
    /// The hour of the day field has most of the time a valid range of **0-23** (inclusive),
    /// however, this range may not always hold true, in-fact in special occasions. For example,
    /// daylight saving hours
    hour, Hour
);

calendar_builder_method!(
    /// The minute of the hour field has a valid range of **0-59** (inclusive)
    minute, Minute
);

calendar_builder_method!(
    /// The second of the minute field has a valid range of **0-59** (inclusive)
    second, Second
);

calendar_builder_method!(
    /// The millisecond of the second field has a valid range of **0-999** (inclusive)
    millisecond, Millisecond
);

#[inline(always)]
fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    (NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap() - chrono::Duration::days(1)).day()
}

#[inline]
fn rebuild_datetime_from_parts(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
) -> DateTime<Local> {
    let day = std::cmp::min(day, last_day_of_month(year, month));
    let naive = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_milli_opt(hour, minute, second, millisecond)
        .expect("Invalid timestamp");

    match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt1, _) => dt1,
        LocalResult::None => {
            let mut candidate = naive;
            for _ in 0..10 {
                candidate += chrono::Duration::minutes(1);
                if let LocalResult::Single(dt) = Local.from_local_datetime(&candidate) {
                    return dt;
                }
            }
            chrono::Utc.from_utc_datetime(&naive).with_timezone(&Local)
        }
    }
}

impl<
    Year: TaskCalendarField + 'static,
    Month: TaskCalendarField + 'static,
    Day: TaskCalendarField + 'static,
    Hour: TaskCalendarField + 'static,
    Minute: TaskCalendarField + 'static,
    Second: TaskCalendarField + 'static,
    Millisecond: TaskCalendarField + 'static,
> TaskSchedule for TaskScheduleCalendar<Year, Month, Day, Hour, Minute, Second, Millisecond>
{
    fn next_after(&self, time: &DateTime<Local>) -> Result<DateTime<Local>, TaskError> {
        let mut fields: [&dyn TaskCalendarField; 7] = [
            &self.millisecond,
            &self.second,
            &self.minute,
            &self.hour,
            &self.day,
            &self.month,
            &self.year,
        ];

        let dates: &mut [u32] = &mut [
            time.timestamp_subsec_millis(),
            time.second(),
            time.minute(),
            time.hour(),
            time.day0(),
            time.month0(),
            time.year() as u32,
        ];

        for (idx, field) in fields.iter_mut().enumerate() {
            field.evaluate(dates, idx)
        }

        let modified = rebuild_datetime_from_parts(
            dates[6] as i32,
            dates[5] + 1,
            dates[4] + 1,
            dates[3],
            dates[2],
            dates[1],
            dates[0],
        );
        Ok(modified)
    }
}
