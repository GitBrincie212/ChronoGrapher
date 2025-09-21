use crate::schedule::TaskSchedule;
use chrono::{
    DateTime, Datelike, Local, LocalResult, Months, NaiveDate, TimeDelta, TimeZone, Timelike,
};
use std::fmt::Debug;
use std::sync::Arc;
use typed_builder::TypedBuilder;

/// Defines a field on the schedule for [`TaskSchedule::Calendar`], by itself it just holds data and how this
/// data is scheduled, it is useful for [`TaskSchedule::Calendar`] only, all fields used in the calendar are
/// zero-based (they start from zero), fields have their own ranges defined, typically:
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
/// A field schedule has 4 variants which it can be in, these are:
/// - **Ignore** ignores the field, instead using the current time's corresponding field
/// - **Every(u32)** tells the scheduler to schedule this field on an interval basis
/// - **Exactly(u32)** tells the scheduler to schedule this field at an exact value
/// - **Custom(Arc<dyn Fn(u32) -> u32 + Send + Sync>)** triggers a custom function to run, where its argument
///   is the current time's field and returns the corresponding field to use (behaves like Exactly but as a function)
///
/// # See
/// - [`TaskSchedule`]
/// - [`crate::scheduler::Scheduler`](scheduler)
#[derive(Clone, Default)]
pub enum TaskCalendarField {
    #[default]
    Ignore,
    Every(u32),
    Exactly(u32),
    Custom(Arc<dyn Fn(u32) -> u32 + Send + Sync>),
}

impl Debug for TaskCalendarField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("CalendarFieldSchedule::{}", &{
            match self {
                Self::Ignore => "Ignore".to_owned(),
                Self::Every(d) => format!("Every({d:?})"),
                Self::Exactly(res) => format!("Exactly({res:?})"),
                Self::Custom(_) => "Custom(...)".to_owned(),
            }
        }))
    }
}

/// [`TaskScheduleCalendar`] is an implementation of the [`TaskSchedule`] trait that allows defining
/// schedules with fine-grained control over individual calendar fields.
///
/// Each field can be configured independently to restrict when the schedule should match.
/// By default, all fields are set to [`TaskCalendarField::Ignore`], which means the field
/// is ignored and instead replaced with the time's fields in [`TaskScheduleCalendar::next_after`] (if
/// there is an exact field after the ignore, then its zero)
///
/// # Examples
///
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
/// # Fields
///
/// - **year** The year field. This is the only unrestricted field and can take any value.
/// - **month** The month field. Valid range: **0–11** (inclusive), where `0 = January`, `11 = December`.
/// - **day** The day-of-month field. Valid range: **0–30** (inclusive). <br />
///   ⚠️ Note: Actual limits depend on the selected month (e.g., February or leap years).
/// - **hour** The hour-of-day field. Valid range: **0–23** (inclusive). <br />
///   ⚠️ Note: Some days may have exceptions due to daylight saving transitions.
/// - **minute** The minute field. Valid range: **0–59** (inclusive).
/// - **second** The second field. Valid range: **0–59** (inclusive).
/// - **millisecond** The millisecond field. Valid range: **0–999** (inclusive).
///
/// # See
/// - [`TaskSchedule`]
/// - [`TaskCalendarField`]
#[derive(TypedBuilder, Clone)]
pub struct TaskScheduleCalendar {
    #[builder(default=TaskCalendarField::Ignore)]
    year: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    month: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    day: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    hour: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    minute: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    second: TaskCalendarField,

    #[builder(default=TaskCalendarField::Ignore)]
    millisecond: TaskCalendarField,
}

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
        .unwrap();

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

impl TaskSchedule for TaskScheduleCalendar {
    fn next_after(
        &self,
        time: &DateTime<Local>,
    ) -> Result<DateTime<Local>, Arc<dyn std::error::Error + 'static>> {
        let mut dates = [
            time.timestamp_subsec_millis(),
            time.second(),
            time.minute(),
            time.hour(),
            time.day0(),
            time.month0(),
            time.year() as u32,
        ];
        let fields = [
            &self.millisecond,
            &self.second,
            &self.minute,
            &self.hour,
            &self.day,
            &self.month,
            &self.year,
        ];
        for (index, &field) in fields.iter().enumerate() {
            let date_field = dates.get_mut(index).unwrap();
            match field {
                TaskCalendarField::Ignore => {}
                TaskCalendarField::Every(d) => {
                    *date_field +=
                        (*d as f64).clamp(u32::MIN as f64, u32::MAX as f64).round() as u32;
                }
                TaskCalendarField::Exactly(res) => {
                    *date_field = *res;
                }
                TaskCalendarField::Custom(func) => {
                    *date_field = func(*date_field);
                }
            }
        }
        let mut modified = rebuild_datetime_from_parts(
            dates[6] as i32,
            dates[5] + 1,
            dates[4] + 1,
            dates[3],
            dates[2],
            dates[1],
            dates[0],
        );
        for (index, &field) in fields.iter().enumerate() {
            if index > 0
                && matches!(fields[index - 1], TaskCalendarField::Exactly(_))
                && !matches!(field, TaskCalendarField::Exactly(_))
                && modified < *time
            {
                match index {
                    0 => {}
                    1 => modified += TimeDelta::seconds(1),
                    2 => modified += TimeDelta::minutes(1),
                    3 => modified += TimeDelta::hours(1),
                    4 => modified += TimeDelta::days(1),
                    5 => modified = modified + Months::new(1),
                    6 => modified = modified + Months::new(12),
                    _ => {}
                };
            }
        }
        Ok(modified)
    }
}
