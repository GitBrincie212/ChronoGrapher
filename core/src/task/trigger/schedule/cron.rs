use crate::errors::{CronError, CronErrorTypes, CronExpressionParserErrors};
use crate::task::schedule::TaskSchedule;
use crate::task::schedule::cron_lexer::{Token, tokenize_fields};
use crate::task::schedule::cron_parser::{AstNode, AstTreeNode, CronParser};
use std::error::Error;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::SystemTime;

const RANGES: [RangeInclusive<u8>; 6] = [
    0..=59u8,
    0..=59u8,
    0..=23u8,
    1u8..=31u8,
    1u8..=12u8,
    1u8..=7u8,
];

const FIELD_NAMES: [&str; 6] = [
    "seconds",
    "minutes",
    "hours",
    "day_of_month",
    "month",
    "day_of_week",
];

fn validate_ast_node(node: &AstNode, field_pos: usize) -> Result<(), CronExpressionParserErrors> {
    let range = &RANGES[field_pos];
    let field_name = FIELD_NAMES[field_pos];

    match &node.kind {
        AstTreeNode::Exact(value) => {
            if !range.contains(value) {
                return Err(CronExpressionParserErrors::ValueOutOfRange {
                    value: *value,
                    field: field_name.to_string(),
                    min: *range.start(),
                    max: *range.end(),
                });
            }
        }

        AstTreeNode::Range(start, end) => {
            let start_val = match &start.kind {
                AstTreeNode::Exact(val) => *val,
                _ => return Err(CronExpressionParserErrors::ExpectedNumber),
            };
            let end_val = match &end.kind {
                AstTreeNode::Exact(val) => *val,
                _ => return Err(CronExpressionParserErrors::ExpectedNumber),
            };

            if start_val > end_val {
                return Err(CronExpressionParserErrors::InvalidRange {
                    start: start_val,
                    end: end_val,
                    field: field_name.to_string(),
                    min: *range.start(),
                    max: *range.end(),
                });
            }

            if !range.contains(&start_val) || !range.contains(&end_val) {
                return Err(CronExpressionParserErrors::InvalidRange {
                    start: start_val,
                    end: end_val,
                    field: field_name.to_string(),
                    min: *range.start(),
                    max: *range.end(),
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
                    kind: AstTreeNode::Wildcard,
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

        let cron_fields: [CronField; 6] = ast
            .iter()
            .map(ast_to_cron_field)
            .collect::<Vec<_>>()
            .try_into()
            .expect("Failed to convert Vec to array");

        Ok(TaskScheduleCron::new(cron_fields))
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, _time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}
