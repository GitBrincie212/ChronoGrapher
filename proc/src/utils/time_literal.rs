use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeInclusive};
use strsim::levenshtein;
use syn::{Expr, ExprLit, Lit};
use syn::parse::{Parse, ParseStream};

pub enum TimeLiteralType {
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

pub struct TimeLiteral {
    pub value: f64,
    pub ty: TimeLiteralType,
}

impl TimeLiteralType {
    pub fn as_usize(&self) -> usize {
        match self {
            TimeLiteralType::Days => 0,
            TimeLiteralType::Hours => 1,
            TimeLiteralType::Minutes => 2,
            TimeLiteralType::Seconds => 3,
            TimeLiteralType::Milliseconds => 4
        }
    }
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
            RangeType::Bounded(range) => range.contains(&num) && *num != 0.0,
            RangeType::Inclusive(range) => range.contains(&num) && *num != 0.0,
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
const TIME_FIELD: [&str; 5] = ["milliseconds", "seconds", "minutes", "hours", "days"];
const SUFFIXES: [&str; 5] = ["ms", "s", "m", "h", "d"];

fn search_suffixes<'a>(target: &str) -> Result<(&'a RangeType, usize), (usize, &'a str)> {
    let mut min_pair = (usize::MAX, "");
    for (idx, suffix) in SUFFIXES.iter().enumerate() {
        if *suffix == target {
            let range = &RANGES[idx];
            return Ok((range, idx));
        }

        let dist = levenshtein(&target, suffix);
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
                            range.to_string(),
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
                let msg = if suffix == "" {
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