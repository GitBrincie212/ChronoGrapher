use crate::schedule::TaskSchedule;
use chrono::{
    DateTime, Datelike, Local, LocalResult, NaiveDate, TimeZone, Timelike,
};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use typed_builder::TypedBuilder;

/// [`TaskCalendarFieldType`] represents the date field type that is being modified, by itself
/// it doesn't hold any data, just what field is being modified. This is used closely with
/// [`TaskCalendarField`] and subsequently [`TaskScheduleCalendar`]
///
/// # Variants
/// The [`TaskCalendarFieldType`] enum includes:
/// - [`TaskCalendarFieldType::YEARS`] for year field
/// - [`TaskCalendarFieldType::MONTHS`] for month field
/// - [`TaskCalendarFieldType::DAYS`] for days field
/// - [`TaskCalendarFieldType::HOURS`] for hours field
/// - [`TaskCalendarFieldType::MINUTES`] for minutes field
/// - [`TaskCalendarFieldType::SECONDS`] for seconds field
/// - [`TaskCalendarFieldType::MILLISECONDS`] for milliseconds field
///
/// # Construction
/// There are no special strings attached, [`TaskCalendarFieldType`] can be constructed with
/// rust's enum initialization easily
///
/// # Trait Implementation(s)
/// There are many traits [`TaskCalendarFieldType`] implements, those being
/// - [`Debug`]
/// - [`Clone`]
/// - [`Copy`]
/// - [`PartialEq`]
/// - [`Eq`]
/// - [`PartialOrd`]
/// - [`Ord`]
///
/// # See Also
/// - [`TaskCalendarField`]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskCalendarFieldType {
    YEARS = 6,
    MONTHS = 5,
    DAYS = 4,
    HOURS = 3,
    MINUTES = 2,
    SECONDS = 1,
    MILLISECONDS = 0
}

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
    fn evaluate(
        &self,
        date_field: &mut u32,
        date_field_type: TaskCalendarFieldType
    );
}

impl<T> TaskCalendarField for T
where
    T: Deref + Send + Sync,
    T::Target: TaskCalendarField
{
    fn evaluate(&self, date_field: &mut u32, date_field_type: TaskCalendarFieldType) {
        self.deref().evaluate(date_field, date_field_type)
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
    fn evaluate(
        &self,
        _date_field: &mut u32,
        _date_field_type: TaskCalendarFieldType
    ) {}
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
    fn evaluate(
        &self,
        date_field: &mut u32,
        _date_field_type: TaskCalendarFieldType
    ) {
        *date_field = self.0
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
    fn evaluate(
        &self,
        date_field: &mut u32,
        _date_field_type: TaskCalendarFieldType
    ) {
        *date_field = date_field.saturating_add(self.0);
    }
}

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
#[derive(TypedBuilder, Clone)]
pub struct TaskScheduleCalendar {
    /// The year field, it is the only unrestricted and can be any value (non-negative)
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    year: Arc<dyn TaskCalendarField>,

    /// The month field has a valid range of **0-11** (inclusive) where `0 = January`, `11 = December`
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    month: Arc<dyn TaskCalendarField>,

    /// The day of the month field has most of the time a valid range of **0-30** (inclusive),
    /// however, this range may not always hold true, in-fact in special occasions. For example,
    /// when the month is set to 2 (February), it is 28 days (and sometimes 29 days on leap
    /// years)
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    day: Arc<dyn TaskCalendarField>,

    /// The hour of the day field has most of the time a valid range of **0-23** (inclusive),
    /// however, this range may not always hold true, in-fact in special occasions. For example,
    /// daylight saving hours
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    hour: Arc<dyn TaskCalendarField>,

    /// The minute of the hour field has a valid range of **0-59** (inclusive)
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    minute: Arc<dyn TaskCalendarField>,

    /// The second of the minute field has a valid range of **0-59** (inclusive)
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    second: Arc<dyn TaskCalendarField>,

    /// The millisecond of the second field has a valid range of **0-999** (inclusive)
    ///
    /// # Default Value
    /// By default, it is set to [`TaskCalendarFieldIdentity`], i.e. It uses the same
    /// date field as the supplied time
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskCalendarFieldIdentity`]
    /// - [`TaskCalendarField`]
    #[builder(default=Arc::new(TaskCalendarFieldIdentity))]
    millisecond: Arc<dyn TaskCalendarField>,
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
        let date_field_types = [
            TaskCalendarFieldType::YEARS,
            TaskCalendarFieldType::MONTHS,
            TaskCalendarFieldType::DAYS,
            TaskCalendarFieldType::HOURS,
            TaskCalendarFieldType::MINUTES,
            TaskCalendarFieldType::SECONDS,
            TaskCalendarFieldType::MILLISECONDS,
        ];
        for (index, &field) in fields.iter().enumerate() {
            let date_field = dates.get_mut(index).unwrap();
            field.evaluate(date_field, date_field_types[index])
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
