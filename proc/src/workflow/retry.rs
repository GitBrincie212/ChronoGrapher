use quote::{quote, ToTokens, TokenStreamExt};
use syn::{bracketed, parenthesized, LitInt, Token};
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::parse::discouraged::Speculative;
use crate::utils::TimeLiteral;
use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};

pub enum JitterType {
    FullJitter,
    EqualJitter,
    DecorrelatedJitter(ValueSource<LitInt>)
}

impl Parse for JitterType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value = input.parse::<syn::Ident>()?;
        match value.to_string().as_str() {
            "full" => Ok(JitterType::FullJitter),
            "equal" => Ok(JitterType::EqualJitter),
            "decorrelated" => {
                let content;
                parenthesized!(content in input);

                let value = content.parse()?;
                Ok(JitterType::DecorrelatedJitter(value))
            }

            _ => {
                Err(syn::Error::new(input.span(), "Unknown jitter type"))
            }
        }
    }
}

pub enum RetryDelay {
    Constant(ValueSource<TimeLiteral>),
    Immediate,
    Custom(syn::Expr),

    Linear {
        factor: syn::Expr,
        start: Option<syn::Expr>,
        clamp: Option<syn::Expr>
    },

    Exponential {
        start: syn::Expr,
        factor: syn::Expr,
        clamp: Option<syn::Expr>
    },

    Jitter {
        jitter_type: JitterType,
        factor: syn::Expr,
        delay: Box<RetryDelay>,
    },
}

impl ToTokens for RetryDelay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expanded = match self {
            RetryDelay::Constant(value) =>
                quote! { Box::new(chronographer::task::frames::retryframe::ConstantBackoffStrategy::new(#value)) },

            RetryDelay::Immediate =>
                quote! { Box::new(chronographer::task::frames::retryframe::ConstantBackoffStrategy::new(std::time::Duration::ZERO)) },

            RetryDelay::Custom(value) => quote! { #value },

            RetryDelay::Linear { start, factor, clamp } => {
                let expanded_clamp = clamp.as_ref().map(|x| quote! {.clamp(#x)});
                let expanded_start = start.as_ref().map(|x| quote! {.start(#x)});

                quote! { Box::new(chronographer::task::frames::retryframe::LinearBackoffStrategy::builder()
                    .factor(#factor)
                    #expanded_start
                    #expanded_clamp
                    .build()
                ) }
            }

            RetryDelay::Exponential { start, factor, clamp } => {
                let expanded_clamp = clamp.as_ref().map(|x| quote! {.clamp(#x)});

                quote! { Box::new(chronographer::task::frames::retryframe::ExponentialBackoffStrategy::builder()
                    .factor(#factor)
                    .start(#start)
                    #expanded_clamp
                    .build()
                ) }
            }
            RetryDelay::Jitter { jitter_type, delay, factor } => {
                let expanded_method = match jitter_type {
                    JitterType::FullJitter => quote! { 
                        chronographer::task::frames::retryframe::JitterBackoffStrategy::new_full(#delay, #factor) 
                    },
                    JitterType::EqualJitter => quote! { 
                        chronographer::task::frames::retryframe::JitterBackoffStrategy::new_equal(#delay, #factor) 
                    },
                    JitterType::DecorrelatedJitter(max) => quote! {
                        chronographer::prelude::new_decorrelated(#delay, #factor, #max)
                    }
                };


                quote! { Box::new(#expanded_method) }
            }
        };

        tokens.append_all(expanded);
    }
}

impl Parse for RetryDelay {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident.to_string() == "immediate" {
                return Ok(RetryDelay::Immediate);
            }

            let content;
            parenthesized!(content in input);

            return match ident.to_string().as_str() {
                "constant" => Ok(Self::Constant(content.parse()?)),
                "linear" => {
                    let mut arg_parser = ArgumentParser::new(&content);
                    let factor = arg_parser.parse_required("factor")?;
                    let start = arg_parser.parse_optional("start")?;
                    let clamp = arg_parser.parse_optional("clamp")?;
                    Ok(Self::Linear { start, factor, clamp })
                }

                "exponential" => {
                    let start: syn::Expr = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let factor: syn::Expr = content.parse()?;
                    Ok(Self::Exponential { start, factor, clamp: None })
                }

                "jitter" => {
                    let jitter_type: JitterType = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let delay = Box::new(content.parse()?);
                    content.parse::<Token![,]>()?;
                    let factor = content.parse()?;
                    content.parse::<Token![,]>()?;
                    Ok(Self::Jitter { jitter_type, factor, delay })
                }

                _ => Err(input.error("Unknown delay value expression"))
            };
        }

        let fork = input.fork();
        match fork.parse() {
            Ok(v) => {
                input.advance_to(&fork);
                Ok(Self::Constant(v))
            }

            Err(_) => Ok(Self::Custom(input.parse()?)),
        }
    }
}

pub enum InnerRetryErrorFilter {
    Patterns(Vec<syn::Pat>),
    Closure(syn::ExprClosure),
    Function(syn::Ident),
    Macro(syn::ExprMacro),
}

fn try_bracketed<'a>(input: &ParseStream<'a>) -> syn::Result<ParseBuffer<'a>> {
    let content;
    bracketed!(content in input);

    Ok(content)
}

fn try_parse_patterns(content: ParseStream) -> syn::Result<InnerRetryErrorFilter> {
    let mut patterns = Vec::new();
    while !content.is_empty() {
        let pat = content.call(syn::Pat::parse_multi)?;
        let _ = content.parse::<Token![,]>();
        patterns.push(pat);
    }

    Ok(InnerRetryErrorFilter::Patterns(patterns))
}

impl InnerRetryErrorFilter {
    fn parse(input: ParseStream, blacklist: bool) -> syn::Result<Self> {
        if blacklist {
            let content;
            bracketed!(content in input);
            return try_parse_patterns(&content);
        }

        let fork = input.fork();

        if fork.peek(syn::Ident) && fork.peek2(Token![!]) {
            let mac: syn::ExprMacro = input.parse()?;
            return Ok(Self::Macro(mac));
        }

        if fork.peek(Token![|]) {
            let closure: syn::ExprClosure = input.parse()?;
            return Ok(Self::Closure(closure));
        }

        if fork.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            return Ok(Self::Function(ident));
        }

        if let Ok(content) = try_bracketed(&input) {
            return try_parse_patterns(&content);
        }

        Err(input.error("Invalid retry error filter"))
    }
}

pub struct RetryErrorFilter {
    inner: InnerRetryErrorFilter,
    blacklist: bool
}

impl Parse for RetryErrorFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut blacklist = false;

        if input.peek(Token![!]) {
            let _ = input.parse::<Token![!]>()?;
            blacklist = true;
        }

        let inner = InnerRetryErrorFilter::parse(input, blacklist)?;
        Ok(RetryErrorFilter { inner, blacklist })
    }
}

pub struct RetryArguments {
    max: ValueSource<LitInt>,
    delay: Option<RetryDelay>,
    when: Option<RetryErrorFilter>
}

impl Parse for RetryArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let max = argument_parser.parse_required("max")?;
        let delay = argument_parser.parse_optional("delay")?;
        let when = argument_parser.parse_optional("when")?;
        Ok(RetryArguments { max, delay, when })
    }
}

impl WorkflowTransform for RetryArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let expanded_backoff = self.delay.as_ref()
            .map(|x| quote! { .backoff(#x) });

        let max = match &self.max {
            ValueSource::Lit(lit) => {
                let Ok(digits) = lit.base10_parse::<u32>() else {
                    return syn::Error::new_spanned(lit, "Retries must be a positive non-zero number")
                        .to_compile_error();
                };

                if digits == 0 {
                    return syn::Error::new_spanned(lit, "Retries must be a positive non-zero number")
                        .to_compile_error();
                }

                quote! { std::num::NonZeroU32::new(#lit).unwrap() }
            }

            ValueSource::Function(func) => quote! { #func },
            ValueSource::Closure(closure) => quote! { #closure },
            ValueSource::Macro(macr) => quote! { #macr }
        };

        let expanded_when = self.when.as_ref()
            .map(|when| {
                match &when.inner {
                    InnerRetryErrorFilter::Patterns(pats) => {
                        let whitelist = if when.blacklist { quote! { ! }} else { quote!() };
                        quote! { .when(|err| {
                            #whitelist matches!(err, Some(#(#pats)|*))
                        }) }
                    }

                    InnerRetryErrorFilter::Function(func) => quote! { .when(#func) },
                    InnerRetryErrorFilter::Closure(closure) => quote! { .when(#closure) },
                    InnerRetryErrorFilter::Macro(macr) => quote! { .when(#macr) },
                }
            });

        quote! {
            chronographer::task::frames::retryframe::RetriableTaskFrame::builder()
                .frame(#toks)
                .retries(#max)
                #expanded_when
                #expanded_backoff
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        quote! { chronographer::task::frames::retryframe::RetriableTaskFrame<#toks> }
    }
}