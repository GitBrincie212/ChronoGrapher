use std::error::Error;
use crate::task::schedule::TaskSchedule;
use chrono::{DateTime, Datelike, Local, LocalResult, NaiveDate, TimeZone, Timelike, Utc};
use std::fmt::Debug;
use std::ops::{Bound, Deref, RangeBounds};
use std::time::SystemTime;

pub trait TaskCalendarField: Send + Sync {
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

#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldIdentity;

impl TaskCalendarField for TaskCalendarFieldIdentity {
    fn evaluate(&self, _date_field: &mut [u32], _idx: usize) {}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldExact(u32);

impl TaskCalendarFieldExact {
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

#[derive(Debug, Clone)]
pub struct TaskCalendarFieldRange<T: TaskCalendarField>(u32, Option<u32>, T);

impl<T: TaskCalendarField> TaskCalendarFieldRange<T> {
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

#[derive(Debug, Default, Clone, Copy)]
pub struct TaskCalendarFieldInterval(u32);

impl TaskCalendarFieldInterval {
    pub fn new(interval: u32) -> Self {
        Self(interval)
    }
}

impl TaskCalendarField for TaskCalendarFieldInterval {
    fn evaluate(&self, date_field: &mut [u32], idx: usize) {
        date_field[idx] = date_field[idx].saturating_add(self.0);
    }
}

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
    year, Year
);

calendar_builder_method!(
    month, Month
);

calendar_builder_method!(
    day, Day
);

calendar_builder_method!(
    hour, Hour
);

calendar_builder_method!(
    minute, Minute
);

calendar_builder_method!(
    second, Second
);

calendar_builder_method!(
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
    fn schedule(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        let time = DateTime::<Utc>::from(now);
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
        Ok(SystemTime::from(modified))
    }
}
