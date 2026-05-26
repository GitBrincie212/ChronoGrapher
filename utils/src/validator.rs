use std::ops::RangeInclusive;

use crate::{cron_parser::{AstNode, AstTreeNode}, error::CronExpressionParserErrors};

const RANGES: [RangeInclusive<u32>; 7] = [
    0..=59,
    0..=59,
    0..=23,
    1..=31,
    1..=12,
    1..=7,
    2026..=2099,
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

pub fn validate_ast_node(node: &AstNode, field_pos: usize) -> Result<(), CronExpressionParserErrors> {
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
