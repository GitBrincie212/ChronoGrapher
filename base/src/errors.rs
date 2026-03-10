use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use thiserror::Error;

pub trait TaskError: Debug + Display + Send + Sync + 'static {
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

impl<T: Debug + Display + Send + Sync + Any> TaskError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[derive(Error, Debug)]
pub enum ConditionalTaskFrameError<T1: TaskError, T2: TaskError> {
    #[error(
        "ConditionalTaskFrame has failed, with the error originating from primary TaskFrame's failure:\n\t{0}"
    )]
    PrimaryFailed(T1),

    #[error(
        "ConditionalTaskFrame has failed, with the error originating from secondary TaskFrame's failure:\n\t{0}"
    )]
    SecondaryFailed(T2),

    #[error("ConditionalTaskFrame has returned false with `error_on_false` enabled")]
    TaskConditionFail,
}

#[derive(Error, Debug)]
pub enum TimeoutTaskFrameError<T: TaskError> {
    #[error(
        "TimeoutTaskFrame has failed, with the error originating from primary TaskFrame's failure:\n\t{0}"
    )]
    Inner(T),

    #[error("TimeoutTaskFrame has timeout with max duration '{0:?}'")]
    Timeout(Duration),
}

#[derive(Error, Debug)]
pub enum DependencyTaskFrameError<T: TaskError> {
    #[error(
        "DependencyTaskFrame has failed, with the error originating from inner TaskFrame's failure:\n\t{0}"
    )]
    Inner(T),

    #[error(
        "DependencyTaskFrame has failed with the error originating from the \"DependentFailBehavior\":\n\t'{0}'"
    )]
    DependenciesInvalidated(Box<dyn TaskError>),
}

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
    InvalidRangeBounds { start: u8, end: u8 },

    #[error("Number `{num}` exceeds expected range (of {start} - {end})")]
    InvalidNumericRange { num: u8, start: u8, end: u8 },

    #[error("Empty field")]
    EmptyField,
}

#[derive(Error, Debug)]
pub enum StandardCoreErrorsCG {
    #[error(
        "Task frame index `{0}` is out of bounds for `{1}` with task frame size `{2}` element(s)"
    )]
    TaskIndexOutOfBounds(usize, String, usize),

    #[error(
        "ConditionalTaskFrame returned false with error_on_false set to true, as such this error returns"
    )]
    TaskConditionFail,

    #[error("Dependencies have not been resolved")]
    TaskDependenciesUnresolved,

    #[error("Invalid CRON expression (parsing failed); Number of fields not in known format")]
    InvalidCronExpression,

    #[cfg(feature = "chrono")]
    #[error("Timedelta supplied is out of range")]
    IntervalTimedeltaOutOfRange,

    #[error("The current SchedulerEngine does not support scheduler instructions")]
    SchedulerInstructionsUnsupported,
}
