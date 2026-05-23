use proc_macro::TokenStream;
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeInclusive};
use quote::quote;
use strsim::levenshtein;
use syn::{parse_macro_input, Expr, ExprLit, Lit, Token};
use syn::parse::{Parse, ParseStream};

pub enum TimeLiteralType {
    Days = 0,
    Hours = 1,
    Minutes = 2,
    Seconds = 3,
    Milliseconds = 4,
}

pub struct TimeLiteral {
    pub value: f64,
    pub ty: TimeLiteralType,
}

macro_rules! parse_as_positive_fraction {
    ($lit: expr, $lit_span: expr, $name: expr) => {{
        let num = $lit.base10_parse::<f64>().map_err(|_| {
            syn::Error::new(
                $lit_span,
                format!("Expected a {} but got \"{}\"", $name, $lit)
            )
        })?;

        if num <= 0f64 {
            return Err(syn::Error::new(
                $lit_span,
                format!("Expected a {} but got \"{}\"", $name, $lit)
            ))
        }

        num
    }};
}

enum RangeType {
    Bounded(Range<f64>),
    Inclusive(RangeInclusive<f64>),
}

impl RangeType {
    fn contains(&self, num: &f64) -> bool {
        match self {
            RangeType::Bounded(range) => range.contains(num) && *num != 0.0,
            RangeType::Inclusive(range) => range.contains(num) && *num != 0.0,
        }
    }
}

impl Display for RangeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeType::Bounded(range) => f.write_fmt(format_args!("{}..{}", range.start, range.end)),
            RangeType::Inclusive(range) => f.write_fmt(format_args!("{}..={}", range.start(), range.end())),
        }
    }
}

const RANGES: [RangeType; 5] = [
    RangeType::Bounded(0.0..1000.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Inclusive(0.0..=31.0)
];
pub const TIME_FIELD: [&str; 5] = ["milliseconds", "seconds", "minutes", "hours", "days"];
const SUFFIXES: [&str; 5] = ["ms", "s", "m", "h", "d"];

fn search_suffixes<'a>(target: &str) -> Result<(&'a RangeType, usize), (usize, &'a str)> {
    let mut min_pair = (usize::MAX, "");
    for (idx, suffix) in SUFFIXES.iter().enumerate() {
        if *suffix == target {
            let range = &RANGES[idx];
            return Ok((range, idx));
        }

        let dist = levenshtein(target, suffix);
        if min_pair.0 > dist {
            min_pair = (dist, *suffix);
        }
    }

    Err(min_pair)
}

impl Parse for TimeLiteral {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit_span = input.cursor().span();
        let Ok(lit) = input.parse::<Expr>() else {
            return Err(syn::Error::new(
                lit_span,
                "Expected a positive integer or float literal but got something else"
            ));
        };

        let (num, suffix) = match lit {
            Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) => {
                let num = parse_as_positive_fraction!(lit, lit_span, "positive integer");

                (num, lit.suffix().to_string())
            }

            Expr::Lit(ExprLit { lit: Lit::Float(lit), .. }) => {
                if lit.to_string().to_ascii_lowercase().contains('e') {
                    return Err(syn::Error::new(
                        lit_span,
                        "Scientific notation is prohibited in use"
                    ))
                }

                let num = parse_as_positive_fraction!(lit, lit_span, "positive float");
                (num, lit.suffix().to_string())
            }

            _ => {
                return Err(syn::Error::new(
                    lit_span,
                    "Expected a positive integer or float literal but got something else"
                ));
            }
        };

        match search_suffixes(&suffix) {
            Ok((range, pos)) => {
                if !range.contains(&num) {
                    return Err(syn::Error::new(
                        lit_span,
                        format!(
                            "Exceeded expected range of {} for \"{}\" time field, got \"{num}\"",
                            range,
                            TIME_FIELD[pos]
                        )
                    ))
                }

                let ty = match pos {
                    0 => TimeLiteralType::Days,
                    1 => TimeLiteralType::Hours,
                    2 => TimeLiteralType::Minutes,
                    3 => TimeLiteralType::Seconds,
                    4 => TimeLiteralType::Milliseconds,
                    _ => unreachable!()
                };

                Ok(TimeLiteral {
                    value: num,
                    ty
                })
            },

            Err((dist, expected)) => {
                let msg = if suffix.is_empty() {
                    "Missing time unit suffix (expected one of: ms, s, m, h, d)".to_string()
                } else if dist < 2 {
                    format!("Unexpected suffix \"{}\", did you mean \"{}\"", suffix, expected)
                } else {
                    format!("Unexpected suffix \"{}\"", suffix)
                };

                Err(syn::Error::new(
                    lit_span,
                    msg
                ))
            }
        }
    }
}

struct Every {
    days: f64,
    hours: f64,
    minutes: f64,
    seconds: f64,
    millis: f64
}

fn extract_expected_values(ptr: usize) -> String {
    if ptr == 0 {
        "nothing".to_string()
    } else if TIME_FIELD[..ptr].len() == 1 {
        format!("\"{}\"", TIME_FIELD[ptr - 1])
    } else {
        format!("either \"{}\"", TIME_FIELD[..ptr].join("\" or \""))
    }
}

fn handle_seperator_format(
    input: &ParseStream,
    is_seperator: bool,
    seperator_format: bool,
    expecting_seperator: &mut bool
)
    -> Result<bool, syn::Error> {
    match (is_seperator, seperator_format, &expecting_seperator) {
        (true, false, _) => {
            Err(syn::Error::new(
                input.span(),
                "Unexpected a seperator \",\""
            ))
        }

        (false, true, true) => {
            Err(syn::Error::new(
                input.span(),
                format!("Expected a seperator (,) but got \"{input}\"")
            ))
        }

        (true, true, true) => {
            let _ = input.parse::<Token![,]>();
            *expecting_seperator = !*expecting_seperator;
            Ok(true)
        }

        (_, _, _) => {
            Ok(false)
        }
    }
}

impl Parse for Every {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ptr = TIME_FIELD.len();
        let mut parts: [f64; 5] = [0.0, 0.0, 0.0, 0.0, 0.0];
        let seperator_format = input.peek2(Token![,]);
        let mut expecting_seperator = false;
        let mut encountered_fractional = false;
        let mut has_modified = false;

        while !input.is_empty() {
            let is_seperator = input.cursor()
                .punct()
                .is_some_and(|(tok, _)| tok.as_char() == ',');

            if handle_seperator_format(&input, is_seperator, seperator_format, &mut expecting_seperator)? {
                continue;
            }

            let lit_span = input.cursor().span();

            expecting_seperator = !expecting_seperator;

            let time_lit = input.parse::<TimeLiteral>()?;
            let is_integer = time_lit.value.round() == time_lit.value;
            if encountered_fractional {
                return Err(syn::Error::new(
                    lit_span,
                    if is_integer {
                        "Unexpected integer followed after fractional part"
                    } else { "Fractional parts are allowed only at the lowest time field" }
                ));
            }

            if !is_integer {
                encountered_fractional = true;
            }

            let pos = time_lit.ty as usize;
            if pos > ptr {
                let expected = extract_expected_values(ptr);

                return Err(syn::Error::new(
                    lit_span,
                    format!("Incorrect time field ordering expected {expected}, got \"{}\"", TIME_FIELD[pos])
                ))
            } else if pos == ptr {
                let expected = extract_expected_values(ptr);

                return Err(syn::Error::new(
                    lit_span,
                    format!("Duplicate time field, expected {expected}, got \"{}\"", TIME_FIELD[pos])
                ))
            }

            ptr = pos;

            parts[pos] = time_lit.value;
            has_modified = true;
        }

        if !has_modified {
            return Err(syn::Error::new(
                input.span(),
                "Expected time field literals got nothing"
            ));
        }

        Ok(Self {
            days: parts[4],
            hours: parts[3],
            minutes: parts[2],
            seconds: parts[1],
            millis: parts[0],
        })
    }
}

#[inline(always)]
pub fn every(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Every);
    let sum = (input.millis / 1000.0)
        + (input.seconds)
        + (input.minutes * 60.0)
        + (input.hours * 3600.0)
        + (input.days * 86400.0);
    
    TokenStream::from(quote! { chronographer::task::schedule::TaskScheduleInterval::from_secs_f64(#sum).unwrap() })
}