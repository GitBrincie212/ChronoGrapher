use crate::errors::{CronError, CronErrorTypes};
use crate::task::schedule::TaskSchedule;
use std::error::Error;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::SystemTime;
use crate::task::schedule::cron_lexer::{tokenize_fields, Token};
use crate::task::schedule::cron_parser::{AstNode, AstTreeNode, CronParser};

const RANGES: [RangeInclusive<u8>; 6] = [
    0..=59u8,
    0..=59u8,
    0..=23u8,
    1u8..=31u8,
    1u8..=12u8,
    1u8..=7u8,
];


#[derive(Clone, Eq, PartialEq, Default)]
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

#[derive(Clone, Eq, PartialEq)]
pub struct TaskScheduleCron {
    seconds: CronField,
    minute: CronField,
    hour: CronField,
    day_of_month: CronField,
    month: CronField,
    day_of_week: CronField,
}

impl TaskScheduleCron {
    pub fn new(cron: [CronField; 6]) -> Self {
        let [seconds, minute, hour, day_of_month, month, day_of_week] = cron;
        Self {
            seconds,
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
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

        let mut ast: [AstNode; 6] = Default::default();
        let mut prev_toks: &[Token] = &tokens[0];
        for (idx, toks) in tokens.iter().enumerate() {
            if toks.len() == 0 {
                ast[idx] = AstNode {
                    start: prev_toks.last().unwrap().start,
                    kind: AstTreeNode::Wildcard
                };
                prev_toks = &toks;
                continue;
            }
            let mut parser_instance = CronParser::new(&toks);
            ast[idx] = parser_instance
                .parse_field()
                .map_err(|error_type| CronError {
                    field_pos: idx,
                    position: (&toks[parser_instance.pos]).start,
                    error_type: CronErrorTypes::Parser(error_type),
                })?;

            prev_toks = &toks;
        }

        todo!()
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, _time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}
