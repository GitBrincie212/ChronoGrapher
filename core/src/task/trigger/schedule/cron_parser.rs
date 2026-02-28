use std::ops::RangeInclusive;
use crate::errors::CronExpressionParserErrors;
use crate::task::schedule::cron_lexer::{Token, TokenType};

#[derive(Default, Clone, Debug)]
pub enum AstTreeNode {
    #[default]
    Wildcard,

    List(Vec<AstTreeNode>),
    Step(Box<AstTreeNode>, u8),
    Range(Box<AstTreeNode>, Box<AstTreeNode>),
    Exact(u8),
    LastOf(Option<u8>),
    Unspecified,
    NthWeekday(u8, u8),
    NearestWeekday(Box<AstTreeNode>),
}

pub struct CronParser<'a> {
    tokens: &'a [Token],
    pub(crate) pos: usize,
}

impl<'a> CronParser<'a> {
    pub(crate) fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(crate) fn parse_field(&mut self) -> Result<AstTreeNode, CronExpressionParserErrors> {
        let node = self.parse_list()?;

        if !self.is_at_end() {
            return Err(CronExpressionParserErrors::UnexpectedToken);
        }

        Ok(node)
    }

    fn parse_list(&mut self) -> Result<AstTreeNode, CronExpressionParserErrors> {
        let mut segments = vec![self.parse_segment()?];

        while self.advanced_if(&TokenType::ListSeparator) {
            segments.push(self.parse_segment()?);
        }

        if segments.len() == 1 {
            return Ok(segments.remove(0));
        }

        Ok(AstTreeNode::List(segments))
    }

    fn parse_segment(&mut self) -> Result<AstTreeNode, CronExpressionParserErrors> {
        let base = self.parse_base()?;

        if self.advanced_if(&TokenType::Step) {
            let step = self.parse_atom()?;

            if let AstTreeNode::Exact(step) = step {
                return Ok(AstTreeNode::Step(Box::new(base), step));
            }

            return Err(CronExpressionParserErrors::ExpectedNumber);
        } else if self.advanced_if(&TokenType::NthWeekday) {
            let AstTreeNode::Exact(val1) = base else {
                return Err(CronExpressionParserErrors::ExpectedNumber);
            };

            let AstTreeNode::Exact(val2) = self.parse_atom()? else {
                return Err(CronExpressionParserErrors::ExpectedNumber);
            };

            return Ok(AstTreeNode::NthWeekday(val1, val2));
        } else if self.advanced_if(&TokenType::NearestWeekday) {
            if let AstTreeNode::Exact(val) = base {
                return Ok(AstTreeNode::NearestWeekday(Box::new(AstTreeNode::Exact(
                    val,
                ))));
            } else if let AstTreeNode::LastOf(None) = base {
                return Ok(AstTreeNode::NearestWeekday(Box::new(AstTreeNode::LastOf(
                    None,
                ))));
            }

            return Err(CronExpressionParserErrors::UnexpectedToken);
        } else if self.advanced_if(&TokenType::Last) {
            let AstTreeNode::Exact(val) = base else {
                return Err(CronExpressionParserErrors::ExpectedNumber);
            };

            return Ok(AstTreeNode::LastOf(Some(val)));
        }

        Ok(base)
    }

    fn parse_base(&mut self) -> Result<AstTreeNode, CronExpressionParserErrors> {
        let start = self.parse_atom()?;

        if self.advanced_if(&TokenType::Minus) {
            let end = self.parse_atom()?;
            return Ok(AstTreeNode::Range(Box::new(start), Box::new(end)));
        }

        Ok(start)
    }

    fn parse_atom(&mut self) -> Result<AstTreeNode, CronExpressionParserErrors> {
        let token = self
            .peek()
            .ok_or(CronExpressionParserErrors::UnexpectedEnd)?;

        match token.token_type {
            TokenType::Wildcard => {
                self.advance();
                Ok(AstTreeNode::Wildcard)
            }

            TokenType::Unspecified => {
                self.advance();
                Ok(AstTreeNode::Unspecified)
            }

            TokenType::Value(val) => {
                self.advance();
                Ok(AstTreeNode::Exact(val))
            }

            TokenType::Last => {
                self.advance();
                Ok(AstTreeNode::LastOf(None))
            }

            _ => Err(CronExpressionParserErrors::ExpectedAtom),
        }
    }

    fn advanced_if(&mut self, expected: &TokenType) -> bool {
        if self.check(expected) {
            self.advance();
            return true;
        }

        false
    }

    fn check(&self, expected: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().unwrap().token_type == *expected
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}