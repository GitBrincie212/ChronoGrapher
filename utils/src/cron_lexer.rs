use proc_macro2::Span;

use crate::errors::CronExpressionLexerErrors;

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
) -> Result<(), (CronExpressionLexerErrors, usize, usize)> {
    // Normalize case once so mixed-case names (e.g. `Mon`, `MoN`) parse the same as `MON`/`mon`.
    let num: u32 = match char_buffer[0..=2].to_ascii_uppercase().as_str() {
        "SUN" if field_pos == 5 => 1,
        "MON" if field_pos == 5 => 2,
        "TUE" if field_pos == 5 => 3,
        "WED" if field_pos == 5 => 4,
        "THU" if field_pos == 5 => 5,
        "FRI" if field_pos == 5 => 6,
        "SAT" if field_pos == 5 => 7,
        "JAN" if field_pos == 4 => 1,
        "FEB" if field_pos == 4 => 2,
        "MAR" if field_pos == 4 => 3,
        "APR" if field_pos == 4 => 4,
        "MAY" if field_pos == 4 => 5,
        "JUN" if field_pos == 4 => 6,
        "JUL" if field_pos == 4 => 7,
        "AUG" if field_pos == 4 => 8,
        "SEP" if field_pos == 4 => 9,
        "OCT" if field_pos == 4 => 10,
        "NOV" if field_pos == 4 => 11,
        "DEC" if field_pos == 4 => 12,
        _ => {
            return Err((
                CronExpressionLexerErrors::UnknownCharacter,
                position,
                field_pos,
            ));
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

pub fn tokenize_from_str(
    s: &str,
) -> Result<[Vec<Token>; 6], (CronExpressionLexerErrors, usize, usize)> {
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
                return Err((
                    CronExpressionLexerErrors::UnknownCharacter,
                    position,
                    field_pos,
                ));
            }

            if field_pos >= 5 {
                return Err((
                    CronExpressionLexerErrors::UnknownFieldFormat,
                    position,
                    field_pos,
                ));
            }

            if tokens[field_pos].is_empty() && field_pos > 0 {
                return Err((CronExpressionLexerErrors::EmptyField, position, field_pos));
            }
            field_pos += 1;
            continue;
        }

        if char_buffer.is_empty() {
            let standalone_token = match char {
                'L' => Some(TokenType::Last),
                'W' if !matches!(chars.peek(), Some((_, 'E' | 'e'))) => {
                    Some(TokenType::NearestWeekday)
                }
                _ => None,
            };

            if let Some(token_type) = standalone_token {
                try_allocate_number(
                    &mut digit_start,
                    &mut current_number,
                    &mut tokens[field_pos],
                );
                tokens[field_pos].push(Token {
                    start: position,
                    token_type,
                    span: None,
                });
                continue;
            }
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
            }
            continue;
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
            '#' => TokenType::NthWeekday,
            _ => {
                return Err((
                    CronExpressionLexerErrors::UnknownCharacter,
                    position,
                    field_pos,
                ));
            }
        };

        tokens[field_pos].push(Token {
            start: position,
            token_type,
            span: None,
        })
    }

    if field_pos != 5 && field_pos != 4 {
        return Err((
            CronExpressionLexerErrors::UnknownFieldFormat,
            s.len() - 1,
            field_pos,
        ));
    }

    if !char_buffer.is_empty() {
        let position = s.len() - char_buffer.len();
        return Err((
            CronExpressionLexerErrors::UnknownCharacter,
            position,
            field_pos,
        ));
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
