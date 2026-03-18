use crate::errors::{CronError, CronErrorTypes, CronExpressionParserErrors};
use crate::task::schedule::TaskSchedule;
use crate::task::schedule::cron_lexer::{Token, tokenize_fields};
use crate::task::schedule::cron_parser::{AstNode, AstTreeNode, CronParser};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use std::error::Error;
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::SystemTime;

const RANGES: [RangeInclusive<u16>; 7] = [
    0..=59u16,
    0..=59u16,
    0..=23u16,
    1u16..=31u16,
    1u16..=12u16,
    1u16..=7u16,
    2026u16..=2099u16,
];

const FIELD_NAMES: [&str; 7] = [
    "seconds",
    "minutes",
    "hours",
    "day_of_month",
    "month",
    "day_of_week",
    "year",
];

fn validate_ast_node(node: &AstNode, field_pos: usize) -> Result<(), CronExpressionParserErrors> {
    let range = &RANGES[field_pos];
    let field_name = FIELD_NAMES[field_pos];

    match &node.kind {
        AstTreeNode::Exact(value) => {
            let value = *value as u16;
            if !range.contains(&value) {
                return Err(CronExpressionParserErrors::ValueOutOfRange {
                    value: value as u8,
                    field: field_name.to_string(),
                    min: *range.start() as u8,
                    max: *range.end() as u8,
                });
            }
        }

        AstTreeNode::Range(start, end) => {
            let start_val = match &start.kind {
                AstTreeNode::Exact(val) => *val as u16,
                _ => return Err(CronExpressionParserErrors::ExpectedNumber),
            };
            let end_val = match &end.kind {
                AstTreeNode::Exact(val) => *val as u16,
                _ => return Err(CronExpressionParserErrors::ExpectedNumber),
            };

            if start_val > end_val {
                return Err(CronExpressionParserErrors::InvalidRange {
                    start: start_val as u8,
                    end: end_val as u8,
                    field: field_name.to_string(),
                    min: *range.start() as u8,
                    max: *range.end() as u8,
                });
            }

            if !range.contains(&start_val) || !range.contains(&end_val) {
                return Err(CronExpressionParserErrors::InvalidRange {
                    start: start_val as u8,
                    end: end_val as u8,
                    field: field_name.to_string(),
                    min: *range.start() as u8,
                    max: *range.end() as u8,
                });
            }
        }

        AstTreeNode::Step(_, step_value) => {
            if *step_value == 0 {
                return Err(CronExpressionParserErrors::InvalidStepValue { step: *step_value });
            }
        }

        AstTreeNode::List(items) => {
            for item in items {
                validate_ast_node(item, field_pos)?;
            }
        }

        AstTreeNode::LastOf(_) => {
            if field_pos != 3 && field_pos != 5 {
                return Err(CronExpressionParserErrors::InvalidLastOperator);
            }
        }

        AstTreeNode::NearestWeekday(_) => {
            if field_pos != 3 {
                return Err(CronExpressionParserErrors::InvalidNearestWeekdayOperator);
            }
        }

        AstTreeNode::NthWeekday(_, nth) => {
            if field_pos != 5 {
                return Err(CronExpressionParserErrors::InvalidNthWeekdayOperator);
            }
            if *nth < 1 || *nth > 5 {
                return Err(CronExpressionParserErrors::InvalidNthWeekday { nth: *nth });
            }
        }

        AstTreeNode::Unspecified => {}

        AstTreeNode::Wildcard => {}
    }

    Ok(())
}

fn ast_to_cron_field(node: &AstNode) -> CronField {
    match &node.kind {
        AstTreeNode::Wildcard => CronField::Wildcard,

        AstTreeNode::Exact(value) => CronField::Exact(*value),

        AstTreeNode::Range(start, end) => {
            let start_val = match &start.kind {
                AstTreeNode::Exact(val) => *val,
                _ => panic!("Range start must be exact value"),
            };
            let end_val = match &end.kind {
                AstTreeNode::Exact(val) => *val,
                _ => panic!("Range end must be exact value"),
            };
            CronField::Range(start_val, end_val)
        }

        AstTreeNode::Step(base, step_value) => {
            let base_field = ast_to_cron_field(base);
            CronField::Step(Box::new(base_field), *step_value)
        }

        AstTreeNode::List(items) => {
            let cron_items: Vec<CronField> = items.iter().map(ast_to_cron_field).collect();
            CronField::List(cron_items)
        }

        AstTreeNode::LastOf(Some(offset)) => CronField::Last(Some(*offset as i8)),
        AstTreeNode::LastOf(None) => CronField::Last(None),

        AstTreeNode::NearestWeekday(base) => {
            let day_val = match &base.kind {
                AstTreeNode::Exact(val) => *val,
                AstTreeNode::LastOf(None) => return CronField::NearestWeekday(0),
                _ => panic!("NearestWeekday base must be exact value or L"),
            };
            CronField::NearestWeekday(day_val)
        }

        AstTreeNode::NthWeekday(day, nth) => CronField::NthWeekday(*day, *nth),

        AstTreeNode::Unspecified => CronField::Unspecified,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum CronField {
    #[default]
    Wildcard,

    Exact(u8),
    Range(u8, u8),
    Step(Box<CronField>, u8),
    List(Vec<CronField>),
    Unspecified,
    Last(Option<i8>),
    NearestWeekday(u8),
    NthWeekday(u8, u8),
}

impl CronField {
    fn matches(&self, value: u8) -> bool {
        match self {
            CronField::Wildcard => true,
            CronField::Exact(v) => *v == value,
            CronField::Range(start, end) => (*start..=*end).contains(&value),
            CronField::Step(base, step) => {
                let start_value = match **base {
                    CronField::Exact(v) => v,
                    CronField::Wildcard => 0,
                    _ => base.min(),
                };
                value >= start_value && (value - start_value).is_multiple_of(*step)
            }
            CronField::List(fields) => fields.iter().any(|f| f.matches(value)),
            CronField::Unspecified => false,
            _ => false,
        }
    }

    fn min(&self) -> u8 {
        match self {
            CronField::Wildcard => 0,
            CronField::Exact(v) => *v,
            CronField::Range(start, _) => *start,
            CronField::Step(base, _) => base.min(),
            CronField::List(fields) => fields.iter().map(|f| f.min()).min().unwrap_or(0),
            CronField::Unspecified => 0,
            CronField::Last(_) | CronField::NearestWeekday(_) | CronField::NthWeekday(_, _) => 1,
        }
    }

    fn max(&self) -> u8 {
        match self {
            CronField::Wildcard => 59,
            CronField::Exact(v) => *v,
            CronField::Range(_, end) => *end,
            CronField::Step(base, step) => {
                let base_max = base.max();
                let base_min = base.min();
                base_max - ((base_max - base_min) % step)
            }
            CronField::List(fields) => fields.iter().map(|f| f.max()).max().unwrap_or(59),
            CronField::Unspecified => 59,
            CronField::Last(_) | CronField::NearestWeekday(_) | CronField::NthWeekday(_, _) => 31,
        }
    }

    fn next_valid(&self, current: u8, field_max: u8) -> Option<u8> {
        if self.matches(current) {
            return Some(current);
        }

        match self {
            CronField::Wildcard => Some(current),
            CronField::Exact(v) => {
                if *v > current {
                    Some(*v)
                } else {
                    None
                }
            }
            CronField::Range(start, end) => {
                if current < *start {
                    Some(*start)
                } else if current <= *end {
                    Some(current)
                } else {
                    None
                }
            }
            CronField::Step(base, step) => {
                let start_value = base.min();
                if current < start_value {
                    Some(start_value)
                } else {
                    let steps_ahead = (current - start_value).div_ceil(*step) * *step;
                    let next_value = start_value + steps_ahead;
                    if next_value <= field_max {
                        Some(next_value)
                    } else {
                        None
                    }
                }
            }
            CronField::List(fields) => {
                let mut candidates: Vec<u8> = fields
                    .iter()
                    .flat_map(|f| {
                        let mut vals = Vec::new();
                        let mut v = f.min();
                        while v <= f.max() && v <= field_max {
                            if f.matches(v) {
                                vals.push(v);
                            }
                            v += 1;
                        }
                        vals
                    })
                    .collect();

                candidates.sort_unstable();
                candidates.into_iter().find(|&v| v >= current)
            }
            _ => None,
        }
    }
}

/// [`TaskScheduleCron`] is a [`TaskSchedule`] used to execute a [Task](crate::task::Task) based on
/// a CRON expression (The [Quartz CRON syntax](https://www.quartz-scheduler.org/documentation/quartz-2.3.0/tutorials/crontrigger.html)).
///
/// # Scheduling Semantics
/// [`TaskScheduleCron`] contains multiple [`CronField`], these are containers which represent the CRON
/// expression at the fundamental level.
///
/// When a schedule occurs, these blocks are used in unison to calculate the new future time (unlike
/// parsing the CRON expression repeatedly just with a new ``DateTime`` which is more performance heavy).
///
/// Typically, the use of [`CronField`] is abstracted away via the [`TaskScheduleCron::from_str`]
/// constructor or via the use of the ``cron!`` macro.
///
/// The CRON implementation is based off how [Quartz CRON](https://www.quartz-scheduler.org/documentation/quartz-2.3.0/tutorials/crontrigger.html)
/// syntax defines it, it is recommended to visit their documentation to learn more on how to use it.
///
/// # Constructor(s)
/// There are two common ways to construct a [`TaskScheduleCron`] instance. The first is via [`TaskScheduleCron::from_str`]
/// for string-based CRON expressions and anything dynamic (value only known at runtime).
///
/// Very useful if the CRON expression is fetched from elsewhere (like a database, an API request... etc.).
/// The main drawback of using this constructor is non compile-time guarantees which easily leads to logic-based errors.
///
/// Alternatively, there is the ``cron!`` macro. Its job is to provide compile-time guarantees the supplied
/// CRON expression is valid as a schedule (in addition it provides a slight performance boost when constructing).
///
/// In most cases its preferred to use the ``cron!`` macro as the main source of construction, however
/// fallback to [`TaskScheduleCron::from_str`] when the expression isn't static and known at compile-time.
///
/// There is a third way to construct a [`TaskScheduleCron`] but it requires the use of manually creating
/// [`CronField`] structs and placing them in an array which is **NOT** recommended (only for providing fancier
/// constructors or macros).
///
/// # Trait Implementation(s)
/// Apart from [`TaskScheduleCron`] implementing the [`TaskSchedule`] trait and [`FromStr`], it implements as well:
/// - [`Debug`]
/// - [`Clone`]
/// - [`PartialEq`]
/// - [`Eq`]
///
/// # Example(s)
/// Using the [`TaskScheduleCron::from_str`] constructor for dynamic-based CRON expressions
/// ```rust
/// use chronographer::prelude::{TaskScheduleCron, TaskTrigger};
/// use std::time::{SystemTime, Duration};
/// # use std::error::Error;
///
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let expr1 = TaskScheduleCron::from_str("* * * * * *")?; // Every second
/// let expr2 = TaskScheduleCron::from_str("0 0 12 * * ?")?; // Every day at 12:00 PM
/// let expr3 = TaskScheduleCron::from_str("0 0/5 14 * * ?")?; // Every 5 minutes from 2:00 PM - 2:55 PM
/// let expr4 = TaskScheduleCron::from_str("0 15 10 ? * MON-FRI")?; // Every Monday, Tuesday, Wednesday, Thursday and Friday at 10:15 AM
/// let expr5 = TaskScheduleCron::from_str("0 15 10 ? * 6L")?; // Every month at last friday at 10:15 AM
/// # Ok(())
/// # }
/// ```
/// In the example we use [`FromStr`] constructor for various CRON expressions (each having a comment next to it
/// explaining its meaning taken from the quartz documentation). In these kinds of CRON expressions it is best
/// to use the ``cron!`` macro which is what the next example shows.
///
/// Using the ``cron!`` macro for static-based CRON expressions
/// ```rust
/// use chronographer::prelude::{cron, TaskScheduleCron, TaskTrigger};
/// use std::time::{SystemTime, Duration};
/// # use std::error::Error;
///
/// # async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
/// let expr1 = cron!("* * * * * *"); // Every second
/// let expr2 = cron!("0 0 12 * * ?"); // Every day at 12:00 PM
/// let expr3 = cron!("0 0/5 14 * * ?"); // Every 5 minutes from 2:00 PM - 2:55 PM
/// let expr4 = cron!("0 15 10 ? * MON-FRI"); // Every Monday, Tuesday, Wednesday, Thursday and Friday at 10:15 AM
/// let expr5 = cron!("0 15 10 ? * 6L"); // Every month at last friday at 10:15 AM
/// # Ok(())
/// # }
/// ```
/// The same example above now uses the ``cron!`` macro for the various CRON expressions. This is generally
/// much preferred overall as even a typo will simply produce a compile-time error (plus slightly faster construction times).
///
/// # Feature Gated?
/// The [cron!](chronographer::prelude::cron) is gated behind the ``macros`` feature which is enabled
/// by default (but can be disabled to not include any macros).
///
/// # See Also
/// - [`TaskScheduleCron::from_str`] - A constructor for dynamic CRON based expressions
/// - [cron!](chronographer::prelude::cron) - A macro with a readable syntax for defining a CRON expression.
/// - [`TaskSchedule`] - The direct implementor of this trait.
/// - [TaskTrigger](crate::task::TaskTrigger) - The general trait which is implemented under the hood.
/// - [`Task`](crate::task::Task) - The main container which the schedule is hosted on.
/// - [`Scheduler`](crate::scheduler::Scheduler) - The side in which it manages the scheduling process of Tasks.
#[derive(Clone, Eq, PartialEq)]
pub struct TaskScheduleCron {
    seconds: CronField,
    minute: CronField,
    hour: CronField,
    day_of_month: CronField,
    month: CronField,
    day_of_week: CronField,
    year: CronField,
}

impl TaskScheduleCron {
    pub fn new(cron: [CronField; 7]) -> Self {
        let [
            seconds,
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
            year,
        ] = cron;
        Self {
            seconds,
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
            year,
        }
    }

    fn next_time_from(&self, current: NaiveDateTime) -> Option<NaiveDateTime> {
        let mut dt = current + Duration::seconds(1);

        loop {
            if !self.matches_year(dt.year() as u16) {
                let next_year = self.next_valid_year(dt.year() as u16)?;
                dt = NaiveDate::from_ymd_opt(next_year as i32, 1, 1)?.and_hms_opt(0, 0, 0)?;
                continue;
            }

            if !self.month.matches(dt.month() as u8) {
                dt = match self.month.next_valid(dt.month() as u8, 12) {
                    Some(next_month) => NaiveDate::from_ymd_opt(dt.year(), next_month as u32, 1)?
                        .and_hms_opt(0, 0, 0)?,
                    None => {
                        let next_year = self.next_valid_year(dt.year() as u16 + 1)?;
                        NaiveDate::from_ymd_opt(next_year as i32, self.month.min() as u32, 1)?
                            .and_hms_opt(0, 0, 0)?
                    }
                };
                continue;
            }

            if !self.matches_day(dt) {
                dt = (dt.date() + Duration::days(1)).and_hms_opt(0, 0, 0)?;
                continue;
            }

            if !self.hour.matches(dt.hour() as u8) {
                dt = match self.hour.next_valid(dt.hour() as u8, 23) {
                    Some(next_hour) => dt.date().and_hms_opt(next_hour as u32, 0, 0)?,
                    None => (dt.date() + Duration::days(1)).and_hms_opt(0, 0, 0)?,
                };
                continue;
            }

            if !self.minute.matches(dt.minute() as u8) {
                dt = match self.minute.next_valid(dt.minute() as u8, 59) {
                    Some(next_minute) => dt.date().and_hms_opt(dt.hour(), next_minute as u32, 0)?,
                    None => {
                        let next_hour = self.hour.next_valid(dt.hour() as u8 + 1, 23);
                        match next_hour {
                            Some(hour) => {
                                dt.date()
                                    .and_hms_opt(hour as u32, self.minute.min() as u32, 0)?
                            }
                            None => (dt.date() + Duration::days(1)).and_hms_opt(0, 0, 0)?,
                        }
                    }
                };
                continue;
            }

            if !self.seconds.matches(dt.second() as u8) {
                dt = match self.seconds.next_valid(dt.second() as u8, 59) {
                    Some(next_second) => {
                        dt.date()
                            .and_hms_opt(dt.hour(), dt.minute(), next_second as u32)?
                    }
                    None => {
                        let next_minute = self.minute.next_valid(dt.minute() as u8 + 1, 59);
                        if let Some(minute) = next_minute {
                            dt.date().and_hms_opt(
                                dt.hour(),
                                minute as u32,
                                self.seconds.min() as u32,
                            )?
                        } else if let Some(hour) = self.hour.next_valid(dt.hour() as u8 + 1, 23) {
                            dt.date().and_hms_opt(
                                hour as u32,
                                self.minute.min() as u32,
                                self.seconds.min() as u32,
                            )?
                        } else {
                            (dt.date() + Duration::days(1)).and_hms_opt(0, 0, 0)?
                        }
                    }
                };
                continue;
            }

            return Some(dt);
        }
    }

    fn matches_year(&self, year: u16) -> bool {
        self.year.matches(year as u8)
    }

    fn next_valid_year(&self, current: u16) -> Option<u16> {
        if current > 2099 {
            return None;
        }
        self.year
            .next_valid(current as u8, 99)
            .map(|y| y as u16 + 2026)
    }

    fn matches_day(&self, dt: NaiveDateTime) -> bool {
        let day_matches = matches!(self.day_of_month, CronField::Unspecified)
            || self.day_of_month.matches(dt.day() as u8);
        let weekday_matches = matches!(self.day_of_week, CronField::Unspecified)
            || self
                .day_of_week
                .matches(dt.weekday().num_days_from_sunday() as u8 + 1);

        let dom_specified = !matches!(self.day_of_month, CronField::Unspecified);
        let dow_specified = !matches!(self.day_of_week, CronField::Unspecified);

        if dom_specified && dow_specified {
            day_matches && weekday_matches
        } else {
            day_matches || weekday_matches
        }
    }
}

impl FromStr for TaskScheduleCron {
    type Err = CronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize_fields(s).map_err(|(error_type, position, field_pos)| CronError {
            field_pos,
            position,
            error_type: CronErrorTypes::Lexer(error_type),
        })?;

        let mut ast: [AstNode; 7] = Default::default();
        let mut prev_toks: &[Token] = &tokens[0];
        for (idx, toks) in tokens.iter().enumerate() {
            if toks.is_empty() {
                ast[idx] = AstNode {
                    start: prev_toks.last().map_or(0, |t| t.start),
                    kind: AstTreeNode::Wildcard,
                };
                prev_toks = toks;
                continue;
            }
            let mut parser_instance = CronParser::new(toks);
            ast[idx] = parser_instance
                .parse_field()
                .map_err(|error_type| CronError {
                    field_pos: idx,
                    position: toks[parser_instance.pos].start,
                    error_type: CronErrorTypes::Parser(error_type),
                })?;

            prev_toks = toks;
        }

        for (field_pos, node) in ast.iter().enumerate() {
            validate_ast_node(node, field_pos).map_err(|error_type| CronError {
                field_pos,
                position: node.start,
                error_type: CronErrorTypes::Parser(error_type),
            })?;
        }

        let day_of_month_unspecified = matches!(ast[3].kind, AstTreeNode::Unspecified);
        let day_of_week_unspecified = matches!(ast[5].kind, AstTreeNode::Unspecified);

        if day_of_month_unspecified && day_of_week_unspecified {
            return Err(CronError {
                field_pos: 3,
                position: ast[3].start,
                error_type: CronErrorTypes::Parser(
                    CronExpressionParserErrors::InvalidUnspecifiedField {
                        field: "day_of_month and day_of_week cannot both be unspecified"
                            .to_string(),
                    },
                ),
            });
        }

        let cron_fields: [CronField; 7] = ast
            .iter()
            .map(ast_to_cron_field)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| CronError {
                field_pos: 0,
                position: 0,
                error_type: CronErrorTypes::Parser(
                    CronExpressionParserErrors::InvalidUnspecifiedField {
                        field: "Failed to convert cron fields to array".to_string(),
                    },
                ),
            })?;

        Ok(TaskScheduleCron::new(cron_fields))
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        let duration = time.duration_since(std::time::UNIX_EPOCH)?;
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();

        let chrono_dt =
            chrono::DateTime::from_timestamp(secs as i64, nanos).ok_or("Invalid timestamp")?;
        let dt = chrono_dt.naive_utc();

        let next_dt = self
            .next_time_from(dt)
            .ok_or("No valid scheduling time found")?;

        let next_timestamp = next_dt.and_utc().timestamp();
        let next_system_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(next_timestamp as u64);

        Ok(next_system_time)
    }
}
