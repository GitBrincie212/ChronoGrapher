use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeInclusive};
use strsim::levenshtein;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{PairsMut, Punctuated};
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, ExprLit, FnArg, Lit, Pat, PatType};

pub const LIFETIME_UNSUPPORTED_ERR: &'static str =
    "Lifetimes are unsupported due to 'static lifetime limitations from async";

pub type ParsedContextArgument = (
    syn::Ident,
    syn::Type,
);

pub type ParsedArguments = (
    Punctuated<proc_macro2::Ident, Comma>,
    Punctuated<syn::Type, Comma>,
);

pub(crate) enum RangeType {
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
            RangeType::Bounded(range) => {
                f.write_fmt(format_args!("{}..{}", range.start, range.end))
            }
            RangeType::Inclusive(range) => {
                f.write_fmt(format_args!("{}..={}", range.start(), range.end()))
            }
        }
    }
}

pub(crate) const TIME_LITERAL_RANGES: [RangeType; 5] = [
    RangeType::Bounded(0.0..1000.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Bounded(0.0..60.0),
    RangeType::Inclusive(0.0..=31.0),
];
pub const TIME_LITERAL_FIELD: [&str; 5] = ["milliseconds", "seconds", "minutes", "hours", "days"];
pub const TIME_LITERAL_SUFFIXES: [&str; 5] = ["ms", "s", "m", "h", "d"];

pub fn extract_annotation<T>(
    attrs: &[Attribute],
    annotation: &str,
    result: &mut Option<T>,
    initializer: impl Fn(TokenStream2) -> syn::Result<T>,
) -> syn::Result<()> {
    for attr in attrs {
        let Some(path) = attr.path().segments.last() else {
            continue;
        };

        if path.ident.to_string() != annotation.to_lowercase() {
            continue;
        }

        if result.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                format!("Cannot use the {} macro annotation twice", annotation.to_lowercase()),
            ));
        }

        let syn::Meta::List(list) = &attr.meta else {
            return Err(syn::Error::new_spanned(
                attr,
                format!("{annotation} annotation expected a list of values"),
            ));
        };

        *result = Some(initializer(list.tokens.clone())?);
    }

    Ok(())
}

pub fn extract_docs(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }

            let syn::Meta::NameValue(nv) = &attr.meta else {
                return None;
            };
            let syn::Expr::Lit(expr_lit) = &nv.value else {
                return None;
            };
            let Lit::Str(lit) = &expr_lit.lit else {
                return None;
            };

            let string = lit.value();
            Some(quote! { #[doc = #string] })
        })
        .collect()
}

pub fn handle_generics_phantom_data(
    name: &syn::Ident,
    fn_sig: &syn::Signature,
) -> syn::Result<(
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<Punctuated<proc_macro2::TokenStream, Comma>>,
)> {
    let generics = &fn_sig.generics;
    let where_clause = &fn_sig.generics.where_clause;
    let mut phantom_data = None;
    let mut impl_end_name = quote! { #name };
    let mut normalized_type_params = None;

    if let Some(lt) = generics.lifetimes().next() {
        return Err(syn::Error::new(lt.span(), LIFETIME_UNSUPPORTED_ERR));
    }

    if !generics.params.is_empty() {
        let phantom_type_params = generics
            .type_params()
            .map(|x| {
                let type_param = &x.ident;
                quote! { #type_param }
            })
            .collect::<Punctuated<_, Comma>>();

        phantom_data = Some(quote! { ( std::marker::PhantomData <( #phantom_type_params )> ) });

        let mut temp = phantom_type_params.clone();
        temp.extend(generics.const_params().map(|x| {
            let type_param = &x.ident;
            quote! { #type_param }
        }));

        normalized_type_params = Some(temp);

        impl_end_name = quote! { #name<#normalized_type_params> #where_clause };
    }

    Ok((impl_end_name, phantom_data, normalized_type_params))
}

pub fn extract_arg_name<'a>(pt: &'a PatType, err: &str) -> syn::Result<&'a proc_macro2::Ident> {
    match &*pt.pat {
        Pat::Ident(pat_ident) => Ok(&pat_ident.ident),
        _ => Err(syn::Error::new_spanned(&pt.pat, err)),
    }
}

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

impl ToTokens for TimeLiteral {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let multiplier = match self.ty {
            TimeLiteralType::Days => 3600f64 * 24f64,
            TimeLiteralType::Hours => 3600f64,
            TimeLiteralType::Minutes => 60f64,
            TimeLiteralType::Seconds => 1f64,
            TimeLiteralType::Milliseconds => 0.01f64,
        };

        let res = self.value * multiplier;
        let expanded = quote! { std::time::Duration::from_secs_f64(#res) };
        tokens.append_all(expanded);
    }
}

macro_rules! parse_as_positive_fraction {
    ($lit: expr, $lit_span: expr, $name: expr) => {{
        let num = $lit.base10_parse::<f64>().map_err(|_| {
            syn::Error::new(
                $lit_span,
                format!("Expected a {} but got \"{}\"", $name, $lit),
            )
        })?;

        if num <= 0f64 {
            return Err(syn::Error::new(
                $lit_span,
                format!("Expected a {} but got \"{}\"", $name, $lit),
            ));
        }

        num
    }};
}

fn search_suffixes<'a>(target: &str) -> Result<(&'a RangeType, usize), (usize, &'a str)> {
    let mut min_pair = (usize::MAX, "");
    for (idx, suffix) in TIME_LITERAL_SUFFIXES.iter().enumerate() {
        if *suffix == target {
            let range = &TIME_LITERAL_RANGES[idx];
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
        let Ok(lit) = input.parse::<syn::Expr>() else {
            return Err(syn::Error::new(
                lit_span,
                "Expected a positive integer or float literal but got something else",
            ));
        };

        let (num, suffix) = match lit {
            syn::Expr::Lit(ExprLit {
                lit: Lit::Int(lit), ..
            }) => {
                let num = parse_as_positive_fraction!(lit, lit_span, "positive integer");

                (num, lit.suffix().to_string())
            }

            syn::Expr::Lit(ExprLit {
                lit: Lit::Float(lit),
                ..
            }) => {
                if lit.to_string().to_ascii_lowercase().contains('e') {
                    return Err(syn::Error::new(
                        lit_span,
                        "Scientific notation is prohibited in use",
                    ));
                }

                let num = parse_as_positive_fraction!(lit, lit_span, "positive float");
                (num, lit.suffix().to_string())
            }

            _ => {
                return Err(syn::Error::new(
                    lit_span,
                    "Expected a positive integer or float literal but got something else",
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
                            range, TIME_LITERAL_FIELD[pos]
                        ),
                    ));
                }

                let ty = match pos {
                    0 => TimeLiteralType::Days,
                    1 => TimeLiteralType::Hours,
                    2 => TimeLiteralType::Minutes,
                    3 => TimeLiteralType::Seconds,
                    4 => TimeLiteralType::Milliseconds,
                    _ => unreachable!(),
                };

                Ok(TimeLiteral { value: num, ty })
            }

            Err((dist, expected)) => {
                let msg = if suffix.is_empty() {
                    "Missing time unit suffix (expected one of: ms, s, m, h, d)".to_string()
                } else if dist < 2 {
                    format!(
                        "Unexpected suffix \"{}\", did you mean \"{}\"",
                        suffix, expected
                    )
                } else {
                    format!("Unexpected suffix \"{}\"", suffix)
                };

                Err(syn::Error::new(lit_span, msg))
            }
        }
    }
}

pub struct TaskFrameConstructor {
    pub ty: syn::TypePath,
    pub inner: syn::Expr,
    pub constructor: Option<syn::Ident>
}

impl TaskFrameConstructor {
    pub fn to_token_type(&self) -> TokenStream2 {
        let ty = &self.ty;
        match self.constructor.as_ref() {
            Some(c) if c == "workflow" => quote! { <#ty as ::chronographer::task::frames::TaskFrame>::Workflow },
            _ => quote! { #ty },
        }
    }

    pub fn to_token_construction(&self) -> TokenStream2 {
        let constructor = &self.inner;
        quote! { #constructor }
    }
}

impl Parse for TaskFrameConstructor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        match &expr {
            syn::Expr::Call(call_expr) => {
                let syn::Expr::Path(syn::ExprPath { path, .. }) = call_expr.func.as_ref() else {
                    return Err(input.error(
                        "Expected an obvious constructor call but got something else",
                    ));
                };

                let segments = &path.segments;
                let segment_len = segments.len();
                if segment_len < 2 {
                    return Err(input.error(
                        "Expected a constructor call such as MyType::new(...)",
                    ));
                }

                let type_path = syn::Path {
                    leading_colon: path.leading_colon,
                    segments: segments.iter()
                        .cloned()
                        .take(segment_len.saturating_sub(1))
                        .collect(),
                };

                let ty = syn::TypePath {
                    qself: None,
                    path: type_path,
                };

                Ok(Self {
                    constructor: Some(segments.last().unwrap().ident.clone()),
                    ty,
                    inner: expr,
                })
            }

            syn::Expr::Path(pt) => {
                let type_path = syn::Path {
                    leading_colon: pt.path.leading_colon,
                    segments: pt.path.segments.clone()
                };

                let ty = syn::TypePath {
                    qself: None,
                    path: type_path,
                };

                Ok(Self {
                    ty,
                    inner: expr,
                    constructor: None
                })
            }

            _ => {
                Err(input.error(
                    "Expected a constructor call such as MyType or MyType::new(...)",
                ))
            }
        }
    }
}

pub fn map_fn_args_pairs(fn_args: &mut PairsMut<FnArg, Comma>) -> syn::Result<ParsedArguments> {
    let mut names = Punctuated::new();
    let mut types = Punctuated::new();
    while let Some(argument) = fn_args.next() {
        match argument.value() {
            FnArg::Typed(pt) => {
                let arg_name = extract_arg_name(&pt, "Expected a simple identifier as an argument name")?;
                let arg_type = &*pt.ty;
                names.push(arg_name.clone());
                types.push(arg_type.clone());
            }

            FnArg::Receiver(recv) => {
                return Err(syn::Error::new_spanned(recv, "Invalid syntax, cannot use self, &self or &mut self"));
            }
        }
    }

    if names.len() == 1 {
        names.pop_punct();
        types.pop_punct();
    }

    Ok((names, types))
}