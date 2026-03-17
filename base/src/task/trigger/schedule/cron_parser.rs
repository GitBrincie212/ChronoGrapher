use crate::errors::CronExpressionParserErrors;
use crate::task::schedule::cron_lexer::{Token, TokenType};

#[derive(Clone, Debug, Default)]
pub(crate) struct AstNode {
    pub start: usize,
    pub kind: AstTreeNode,
}

#[derive(Default, Clone, Debug)]
pub(crate) enum AstTreeNode {
    #[default]
    Wildcard,

    List(Vec<AstNode>),
    Step(Box<AstNode>, u8),
    Range(Box<AstNode>, Box<AstNode>),
    Exact(u8),
    LastOf(Option<u8>),
    Unspecified,
    NthWeekday(u8, u8),
    NearestWeekday(Box<AstNode>),
}

pub(crate) struct CronParser<'a> {
    tokens: &'a [Token],
    pub(crate) pos: usize,
}

impl<'a> CronParser<'a> {
    pub(crate) fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(crate) fn parse_field(&mut self) -> Result<AstNode, CronExpressionParserErrors> {
        let node = self.parse_list()?;

        if !self.is_at_end() {
            return Err(CronExpressionParserErrors::UnexpectedToken);
        }

        Ok(node)
    }

    fn parse_list(&mut self) -> Result<AstNode, CronExpressionParserErrors> {
        let mut segments = vec![self.parse_segment()?];

        while self.advanced_if(&TokenType::ListSeparator) {
            segments.push(self.parse_segment()?);
        }

        if segments.len() == 1 {
            return Ok(segments.remove(0));
        }

        Ok(AstNode {
            start: segments[0].start,
            kind: AstTreeNode::List(segments),
        })
    }

    fn parse_segment(&mut self) -> Result<AstNode, CronExpressionParserErrors> {
        let base = self.parse_base()?;

        if let Some(step) = self.try_parse_step(&base)? {
            return Ok(step);
        } else if let Some(nth_weekday) = self.try_parse_nth_weekday(&base)? {
            return Ok(nth_weekday);
        } else if let Some(nereast_weekday) = self.try_parse_nearest_weekday(&base)? {
            return Ok(nereast_weekday);
        } else if let Some(lastof) = self.try_parse_lastof(&base)? {
            return Ok(lastof);
        }

        Ok(base)
    }

    fn try_parse_step(
        &mut self,
        base: &AstNode,
    ) -> Result<Option<AstNode>, CronExpressionParserErrors> {
        if self.advanced_if(&TokenType::Step) {
            let step = self.parse_atom()?;

            if let AstTreeNode::Exact(val) = step.kind {
                return Ok(Some(AstNode {
                    start: base.start,
                    kind: AstTreeNode::Step(Box::new(base.clone()), val),
                }));
            }

            return Err(CronExpressionParserErrors::ExpectedNumber);
        }

        Ok(None)
    }

    fn try_parse_nth_weekday(
        &mut self,
        base: &AstNode,
    ) -> Result<Option<AstNode>, CronExpressionParserErrors> {
        if self.advanced_if(&TokenType::NthWeekday) {
            let AstTreeNode::Exact(val1) = base.kind else {
                return Err(CronExpressionParserErrors::ExpectedNumber);
            };

            let node = self.parse_atom()?;

            let AstTreeNode::Exact(val2) = node.kind else {
                return Err(CronExpressionParserErrors::ExpectedNumber);
            };

            return Ok(Some(AstNode {
                start: base.start,
                kind: AstTreeNode::NthWeekday(val1, val2),
            }));
        }

        Ok(None)
    }

    fn try_parse_nearest_weekday(
        &mut self,
        base: &AstNode,
    ) -> Result<Option<AstNode>, CronExpressionParserErrors> {
        if self.advanced_if(&TokenType::NearestWeekday) {
            if let AstTreeNode::Exact(_) = base.kind {
                return Ok(Some(AstNode {
                    start: base.start,
                    kind: AstTreeNode::NearestWeekday(Box::new(base.clone())),
                }));
            }

            if let AstTreeNode::LastOf(None) = base.kind {
                return Ok(Some(AstNode {
                    start: base.start,
                    kind: AstTreeNode::NearestWeekday(Box::new(base.clone())),
                }));
            }

            return Err(CronExpressionParserErrors::UnexpectedToken);
        }

        Ok(None)
    }

    fn try_parse_lastof(
        &mut self,
        base: &AstNode,
    ) -> Result<Option<AstNode>, CronExpressionParserErrors> {
        if self.advanced_if(&TokenType::Last) {
            let last_node = match base.kind {
                AstTreeNode::Exact(val) => Some(val),
                AstTreeNode::LastOf(None) => None,
                _ => return Err(CronExpressionParserErrors::ExpectedNumber),
            };

            return Ok(Some(AstNode {
                start: base.start,
                kind: AstTreeNode::LastOf(last_node),
            }));
        }

        Ok(None)
    }

    fn parse_base(&mut self) -> Result<AstNode, CronExpressionParserErrors> {
        let atom = self.parse_atom()?;

        if self.advanced_if(&TokenType::Minus) {
            let end = self.parse_atom()?;

            return match atom.kind {
                AstTreeNode::LastOf(None) => {
                    if let AstTreeNode::Exact(val) = end.kind {
                        Ok(AstNode {
                            start: atom.start,
                            kind: AstTreeNode::LastOf(Some(val)),
                        })
                    } else {
                        Err(CronExpressionParserErrors::ExpectedNumber)
                    }
                }
                _ => Ok(AstNode {
                    start: atom.start,
                    kind: AstTreeNode::Range(Box::new(atom), Box::new(end)),
                }),
            };
        }

        Ok(atom)
    }

    fn parse_atom(&mut self) -> Result<AstNode, CronExpressionParserErrors> {
        let token = self
            .peek()
            .ok_or(CronExpressionParserErrors::UnexpectedEnd)?;

        let start = token.start;

        match token.token_type {
            TokenType::Wildcard => {
                self.advance();
                Ok(AstNode {
                    start,
                    kind: AstTreeNode::Wildcard,
                })
            }

            TokenType::Unspecified => {
                self.advance();
                Ok(AstNode {
                    start,
                    kind: AstTreeNode::Unspecified,
                })
            }

            TokenType::Value(val) => {
                self.advance();
                Ok(AstNode {
                    start,
                    kind: AstTreeNode::Exact(val),
                })
            }

            TokenType::Last => {
                self.advance();
                Ok(AstNode {
                    start,
                    kind: AstTreeNode::LastOf(None),
                })
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
