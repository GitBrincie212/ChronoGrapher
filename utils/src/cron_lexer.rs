use proc_macro2::{Span, TokenTree};

use crate::error::CronLexerError;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Value(u32),
    Minus,
    Wildcard,
    ListSeparator,
    Unspecified,
    Step,
    Last,
    NearestWeekday,
    NthWeekday,
}

#[derive(Debug)]
pub struct Token {
    pub start: usize,
    pub token_type: TokenType,
    pub span: Option<Span>,
}

fn constant_to_numeric(
    char_buffer: &mut String,
    field_pos: usize,
    position: usize,
    tokens: &mut Vec<Token>,
) -> Result<(), (CronLexerError, usize, usize)> {
    let num: u32 = match &char_buffer[0..=2] {
        "SUN" | "sun" if field_pos == 3 => 1,
        "MON" | "mon" if field_pos == 3 => 2,
        "TUE" | "tue" if field_pos == 3 => 3,
        "WED" | "wed" if field_pos == 3 => 4,
        "THU" | "thu" if field_pos == 3 => 5,
        "FRI" | "fri" if field_pos == 3 => 6,
        "SAT" | "sat" if field_pos == 3 => 7,
        "JAN" | "jan" if field_pos == 5 => 1,
        "FEB" | "feb" if field_pos == 5 => 2,
        "MAR" | "mar" if field_pos == 5 => 3,
        "APR" | "apr" if field_pos == 5 => 4,
        "MAY" | "may" if field_pos == 5 => 5,
        "JUN" | "jun" if field_pos == 5 => 6,
        "JUL" | "jul" if field_pos == 5 => 7,
        "AUG" | "aug" if field_pos == 5 => 8,
        "SEP" | "sep" if field_pos == 5 => 9,
        "OCT" | "oct" if field_pos == 5 => 10,
        "NOV" | "nov" if field_pos == 5 => 11,
        "DEC" | "dec" if field_pos == 5 => 12,
        _ => {
            return Err((CronLexerError::UnknownCharacter, position, field_pos));
        }
    };

    tokens.push(Token {
        start: position - 3,
        token_type: TokenType::Value(num),
        span: None,
    });
    char_buffer.clear();
    Ok(())
}

fn try_allocate_number(
    digit_start: &mut Option<usize>,
    current_number: &mut u32,
    tokens: &mut Vec<Token>,
) {
    if let Some(start) = digit_start {
        tokens.push(Token {
            start: *start,
            token_type: TokenType::Value(*current_number),
            span: None,
        });
        *current_number = 0;
        *digit_start = None;
    }
}

pub fn tokenize_from_str(s: &str) -> Result<[Vec<Token>; 6], (CronLexerError, usize, usize)> {
    let mut tokens: [Vec<Token>; 6] = [const { Vec::new() }; 6];
    let mut current_number = 0u32;
    let mut field_pos = 0;
    let mut char_buffer: String = String::with_capacity(3);
    let mut chars = s.chars().enumerate().peekable();
    let mut digit_start: Option<usize> = None;
    while let Some((position, char)) = chars.next() {
        if char == ' ' {
            try_allocate_number(
                &mut digit_start,
                &mut current_number,
                &mut tokens[field_pos],
            );
            digit_start = None;
            current_number = 0;

            if char_buffer.len() == 3 {
                constant_to_numeric(
                    &mut char_buffer,
                    field_pos,
                    position,
                    &mut tokens[field_pos],
                )?;
            } else if !char_buffer.is_empty() {
                return Err((CronLexerError::UnknownCharacter, position, field_pos));
            }

            if field_pos >= 5 {
                return Err((CronLexerError::UnknownFieldFormat, position, field_pos));
            }

            if tokens[field_pos].is_empty() && field_pos > 0 {
                return Err((CronLexerError::EmptyField, position, field_pos));
            }
            field_pos += 1;
            continue;
        }

        if char.is_alphabetic() || !char_buffer.is_empty() {
            char_buffer.push(char);
            if char_buffer.len() == 3 {
                constant_to_numeric(
                    &mut char_buffer,
                    field_pos,
                    position,
                    &mut tokens[field_pos],
                )?;
                continue;
            }
        }

        if char.is_ascii_digit() {
            digit_start = Some(position);
            current_number = current_number * 10 + ((char as u8 - b'0') as u32);
            continue;
        }

        try_allocate_number(
            &mut digit_start,
            &mut current_number,
            &mut tokens[field_pos],
        );

        let token_type = match char {
            '-' => TokenType::Minus,
            '*' => TokenType::Wildcard,
            ',' => TokenType::ListSeparator,
            '?' => TokenType::Unspecified,
            '/' => TokenType::Step,
            'L' => {
                char_buffer.clear();
                TokenType::Last
            }
            '#' => TokenType::NthWeekday,
            'W' if !matches!(chars.peek(), Some((_, 'E' | 'e'))) => {
                char_buffer.clear();
                TokenType::NearestWeekday
            }
            _ => {
                return Err((CronLexerError::UnknownCharacter, position, field_pos));
            }
        };

        tokens[field_pos].push(Token {
            start: position,
            token_type,
            span: None,
        })
    }

    if field_pos != 5 && field_pos != 4 {
        return Err((CronLexerError::UnknownFieldFormat, s.len() - 1, field_pos));
    }

    if !char_buffer.is_empty() {
        let position = s.len() - char_buffer.len();
        return Err((CronLexerError::UnknownCharacter, position, field_pos));
    }

    if let Some(start) = digit_start {
        tokens[field_pos].push(Token {
            start,
            token_type: TokenType::Value(current_number),
            span: None,
        });
    }

    Ok(tokens)
}

pub fn tokenize_from_tokens(
    input: proc_macro2::TokenStream,
) -> Result<[Vec<Token>; 6], (CronLexerError, proc_macro2::Span)> {
    let mut tokens: [Vec<Token>; 6] = std::array::from_fn(|_| Vec::new());
    let mut iter = input.into_iter().peekable();
    let mut field_pos = 0;

    while let Some(tt) = iter.next() {
        let tt_span = tt.span();
        match tt {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                let val: u32 = s
                    .parse()
                    .map_err(|_| (CronLexerError::UnknownCharacter, lit.span()))?;

                tokens[field_pos].push(Token {
                    start: 0,
                    token_type: TokenType::Value(val),
                    span: Some(lit.span()),
                })
            }
            TokenTree::Ident(ident) => {
                let s = ident.to_string();
                let token_type = match s.to_uppercase().as_str() {
                    "L" => TokenType::Last,
                    "W" => TokenType::NearestWeekday,
                    "SUN" => TokenType::Value(1),
                    "MON" => TokenType::Value(2),
                    "TUE" => TokenType::Value(3),
                    "WED" => TokenType::Value(4),
                    "THU" => TokenType::Value(5),
                    "FRI" => TokenType::Value(6),
                    "SAT" => TokenType::Value(7),
                    "JAN" => TokenType::Value(1),
                    "FEB" => TokenType::Value(2),
                    "MAR" => TokenType::Value(3),
                    "APR" => TokenType::Value(4),
                    "MAY" => TokenType::Value(5),
                    "JUN" => TokenType::Value(6),
                    "JUL" => TokenType::Value(7),
                    "AUG" => TokenType::Value(8),
                    "SEP" => TokenType::Value(9),
                    "OCT" => TokenType::Value(10),
                    "NOV" => TokenType::Value(11),
                    "DEC" => TokenType::Value(12),
                    _ => return Err((CronLexerError::UnknownCharacter, ident.span())),
                };

                tokens[field_pos].push(Token {
                    start: 0,
                    token_type,
                    span: Some(ident.span()),
                });
            }
            TokenTree::Punct(punct) => {
                let token_type = match punct.as_char() {
                    '*' => TokenType::Wildcard,
                    '-' => TokenType::Minus,
                    ',' => TokenType::ListSeparator,
                    '/' => TokenType::Step,
                    '#' => TokenType::NthWeekday,
                    '?' => TokenType::Unspecified,
                    _ => return Err((CronLexerError::UnknownCharacter, punct.span())),
                };
                tokens[field_pos].push(Token {
                    start: 0,
                    token_type,
                    span: Some(punct.span()),
                })
            }
            TokenTree::Group(_) => {}
        }
        if let Some(next) = iter.peek() {
            if tt_span.end().column < next.span().start().column - 1
                || tt_span.end().line != next.span().start().line
            {
                field_pos += 1;
            }
        }
    }

    Ok(tokens)
}
