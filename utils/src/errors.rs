use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub struct CronError {
    pub field_pos: usize,
    pub position: usize,
    pub error_type: CronErrorTypes,
}

impl Display for CronError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let field_type = match self.field_pos {
            0 => "Seconds",
            1 => "Minutes",
            2 => "Hours",
            3 => "Day Of Month",
            4 => "Month",
            5 => "Day Of Week",
            _ => "UNKNOWN",
        };

        f.write_fmt(format_args!(
            "{}\n\tAt `{field_type}` field and position {}",
            self.error_type, self.position
        ))
    }
}

#[derive(Error, Debug)]
pub enum CronErrorTypes {
    #[error("ParserError: {0}")]
    Parser(CronExpressionParserErrors),

    #[error("LexerError: {0}")]
    Lexer(CronExpressionLexerErrors),
}

#[derive(Error, Debug)]
pub enum CronExpressionParserErrors {
    #[error("Invalid use of list seperator, trialing seperator found")]
    TrialingListSeperator,

    #[error("Invalid use of the step operator, too many subsequent steps found")]
    TooManySteps,

    #[error("Invalid use of list seperator, trialing step found")]
    TrialingStep,

    #[error("Undefined use of the symbol `-`")]
    UndefinedUseOfMinus,

    #[error("Unexpected token sequence found")]
    UnexpectedToken,

    #[error("Expected one or more tokens, found an abrupt end")]
    UnexpectedEnd,

    #[error("Expected atom operator but got something else")]
    ExpectedAtom,

    #[error("Expected number but got something else")]
    ExpectedNumber,

    #[error("Value {value} is out of range for {field} field (expected {min}-{max})")]
    ValueOutOfRange {
        value: u32,
        field: String,
        min: u32,
        max: u32,
    },

    #[error("Invalid range {start}-{end} for {field} field (expected {min}-{max})")]
    InvalidRange {
        start: u32,
        end: u32,
        field: String,
        min: u32,
        max: u32,
    },

    #[error("Step value {step} must be greater than 0")]
    InvalidStepValue { step: u32 },

    #[error("Nth weekday {nth} is out of range (expected 1-5)")]
    InvalidNthWeekday { nth: u32 },

    #[error("Field '{field}' cannot be unspecified in this context")]
    InvalidUnspecifiedField { field: String },

    #[error("L (last) operator is only valid for day_of_month and day_of_week fields")]
    InvalidLastOperator,

    #[error("W (nearest weekday) operator is only valid for day_of_month field")]
    InvalidNearestWeekdayOperator,

    #[error("# (nth weekday) operator is only valid for day_of_week field")]
    InvalidNthWeekdayOperator,
}

#[derive(Error, Debug)]
pub enum CronExpressionLexerErrors {
    #[error("Number of fields not in known format")]
    UnknownFieldFormat,

    #[error("Unknown character")]
    UnknownCharacter,

    #[error("Invalid use of range operator")]
    InvalidRange,

    #[error("Invalid use of wildcard operand")]
    InvalidWildcard,

    #[error("Invalid use of list seperator")]
    InvalidListSeperator,

    #[error("Use of non-numeric operands / operations inside list")]
    NonNumericOperatorUse,

    #[error("Undefined range, minimum bound is higher than maximum bound ({start} >= {end})")]
    InvalidRangeBounds { start: u32, end: u32 },

    #[error("Number `{num}` exceeds expected range (of {start} - {end})")]
    InvalidNumericRange { num: u32, start: u32, end: u32 },

    #[error("Empty field")]
    EmptyField,
}
