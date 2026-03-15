use crate::errors::CronExpressionLexerErrors;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenType {
    Value(u8),
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
pub(crate) struct Token {
    pub(crate) start: usize,
    pub(crate) token_type: TokenType,
}

fn constant_to_numeric(
    char_buffer: &mut String,
    field_pos: usize,
    position: usize,
    tokens: &mut Vec<Token>,
) -> Result<(), (CronExpressionLexerErrors, usize, usize)> {
    let num: u8 = match &char_buffer[0..=2] {
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
    });
    char_buffer.clear();
    Ok(())
}

fn try_allocate_number(
    digit_start: &mut Option<usize>,
    current_number: &mut u8,
    tokens: &mut Vec<Token>,
) {
    if let Some(start) = digit_start {
        tokens.push(Token {
            start: *start,
            token_type: TokenType::Value(*current_number),
        });
        *current_number = 0;
        *digit_start = None;
    }
}

pub(crate) fn tokenize_fields(s: &str) -> Result<[Vec<Token>; 6], (CronExpressionLexerErrors, usize, usize)> {
    let mut tokens: [Vec<Token>; 6] = [const { Vec::new() }; 6];
    let mut current_number = 0u8;
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
            current_number = current_number * 10 + (char as u8 - b'0');
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
        });
    }

    Ok(tokens)
}