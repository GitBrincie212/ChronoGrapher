use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Token};
use syn::parse::{Parse, ParseStream};
use crate::utils::time_literal::{TimeLiteral, TIME_FIELD};

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

            let pos = time_lit.ty.as_usize();
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
    
    TokenStream::from(quote! { chronographer::task::interval::TaskScheduleInterval::from_secs_f64(#sum) })
}