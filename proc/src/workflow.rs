pub mod retry;
pub mod utils;
pub mod timeout;
pub mod delay;
pub mod fallback;
pub mod threshold;
pub mod condition;
mod dependency;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{parenthesized, Token};
use syn::__private::TokenStream2;
use syn::punctuated::Punctuated;
use crate::workflow::condition::ConditionArguments;
use crate::workflow::delay::DelayArguments;
use crate::workflow::dependency::DependencyArguments;
use crate::workflow::fallback::FallbackArguments;
use crate::workflow::retry::RetryArguments;
use crate::workflow::threshold::ThresholdArguments;
use crate::workflow::timeout::TimeoutArguments;
use crate::workflow::utils::WorkflowTransform;

pub struct WorkflowAnnotation;

impl WorkflowAnnotation {
    pub fn translate(
        initial_creation: TokenStream2,
        initial_type: TokenStream2,
        attrs: Punctuated<syn::Expr, Token![,]>
    ) -> syn::Result<(TokenStream2, TokenStream2)> {
        let mut wrappers: Vec<Box<dyn WorkflowTransform>> = Vec::new();

        if attrs.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Expected at least one workflow primitive"
            ))
        }

        for inner in attrs {
            let syn::Expr::Call(call_expr) = inner else {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Expected at least one workflow primitive"
                ));
            };

            let syn::Expr::Path(path_expr) = call_expr.func.as_ref() else {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Expected a simple identifier"
                ));
            };

            let segments = &path_expr.path.segments;
            if segments.len() != 1 {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Expected a workflow primitive identifier"
                ));
            }

            let args = call_expr.args;
            let content = quote! { #args };
            match segments.last().unwrap().ident.to_string().as_str() {
                "retry" => wrappers.push(Box::new(RetryArguments::parse.parse2(content)?)),
                "delay" => wrappers.push(Box::new(DelayArguments::parse.parse2(content)?)),
                "timeout" => wrappers.push(Box::new(TimeoutArguments::parse.parse2(content)?)),
                "fallback" => wrappers.push(Box::new(FallbackArguments::parse.parse2(content)?)),
                "threshold" => wrappers.push(Box::new(ThresholdArguments::parse.parse2(content)?)),
                "dependency" => wrappers.push(Box::new(DependencyArguments::parse.parse2(content)?)),
                // "condition" => wrappers.push(Box::new(ConditionArguments::parse(&content)?)),

                _ => return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Unknown workflow primitive"
                ))
            }
        }

        let mut expanded_creation = quote! { #initial_creation };
        let mut expanded_alias = quote! { #initial_type };
        for primitive in &wrappers {
            expanded_creation = primitive.transform(expanded_creation);
            expanded_alias = primitive.get_type(expanded_alias);
        }
        
        Ok((expanded_creation, expanded_alias))
    }
}

pub fn workflow(_attrs: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "Workflow attribute is unsupported outside of Tasks and TaskFrames (via the respective macros)"
    ).to_compile_error().into()
}