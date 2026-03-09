use proc_macro::TokenStream;
use std::ops::RangeInclusive;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, LitInt, Token};
use strsim::levenshtein;
use syn::parse::{Parse, ParseStream};

#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn greet() -> String {
                my_library::hello(stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}

struct Interval {
    days: u16,
    hours: u16,
    minutes: u16,
    seconds: u16,
    millis: u16
}

const RANGES: [RangeInclusive<u16>; 5] = [0..=999, 0..=59, 0..=59, 0..=59, 0..=31];
const TIME_FIELD: [&str; 5] = ["milliseconds", "seconds", "minutes", "hours", "days"];
const SUFFIXES: [&str; 5] = ["ms", "s", "m", "h", "d"];

fn extract_expected_values(ptr: usize) -> String {
    if ptr == 0 {
        "nothing".to_string()
    } else if TIME_FIELD[..ptr].len() == 1 {
        format!("\"{}\"", TIME_FIELD[ptr - 1])
    } else {
        format!("either \"{}\"", TIME_FIELD[..ptr].join("\" or \""))
    }
}

fn search_suffixes<'a>(lit: &LitInt) -> Result<(&'a RangeInclusive<u16>, usize), (usize, &'a str)> {
    let mut min_pair = (usize::MAX, "");
    for (idx, suffix) in SUFFIXES.iter().enumerate() {
        if *suffix == lit.suffix() {
            let range = &RANGES[idx];
            return Ok((range, idx));
        }

        let dist = levenshtein(&lit.suffix(), suffix);
        if min_pair.0 > dist {
            min_pair = (dist, *suffix);
        }
    }

    Err(min_pair)
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

impl Parse for Interval {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ptr = SUFFIXES.len();
        let mut parts: [u16; 5] = [0, 0, 0, 0, 0];
        let seperator_format = input.peek2(Token![,]);
        let mut expecting_seperator = false;

        while !input.is_empty() {
            let is_seperator = input.cursor()
                .punct()
                .is_some_and(|(tok, _)| tok.as_char() == ',');

            if handle_seperator_format(&input, is_seperator, seperator_format, &mut expecting_seperator)? {
                continue;
            }

            expecting_seperator = !expecting_seperator;

            let lit_span = input.cursor().span();
            let Ok(lit) = input.parse::<LitInt>() else {
                return Err(syn::Error::new(
                    lit_span,
                    "Expected an unsigned integer but got something else"
                ));
            };

            let Ok(num) = lit.base10_parse::<u16>() else {
                return Err(syn::Error::new(
                    lit_span,
                    format!("Expected an unsigned integer (u16) but got \"{lit}\"")
                ))
            };

            match search_suffixes(&lit) {
                Ok((range, pos)) => {
                    if !range.contains(&num) {
                        return Err(syn::Error::new(
                            lit_span,
                            format!(
                                "Exceeded expected range of {}..={} for \"{}\" time field, got \"{num}\"",
                                range.start(),
                                range.end(),
                                TIME_FIELD[pos]
                            )
                        ))
                    }

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

                    parts[pos] = num;

                    continue;
                },

                Err((dist, expected)) => {
                    let msg = if dist < 2 {
                        format!("Unexpected suffix \"{}\", did you mean \"{}\"", lit.suffix(), expected)
                    } else if lit.suffix() == "" {
                        "Missing time unit suffix (expected one of: ms, s, m, h, d)".to_string()
                    } else {
                        format!("Unexpected suffix \"{}\"", lit.suffix())
                    };

                    return Err(syn::Error::new(
                        lit_span,
                        msg
                    ));
                }
            }
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

#[proc_macro]
pub fn interval(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Interval);
    let sum = (input.millis as f64 / 1000.0)
        + (input.seconds as f64)
        + (input.minutes as f64 * 60.0)
        + (input.hours as f64 * 3600.0)
        + (input.days as f64 * 86400.0);

    TokenStream::from(quote! { chronographer::task::interval::TaskScheduleInterval::from_secs_f64(#sum) })
}
