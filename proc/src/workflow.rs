pub mod condition;
pub mod delay;
mod dependency;
pub mod fallback;
pub mod retry;
pub mod threshold;
pub mod timeout;
pub mod utils;

use crate::workflow::condition::ConditionArguments;
use crate::workflow::delay::DelayArguments;
use crate::workflow::dependency::DependencyArguments;
use crate::workflow::fallback::FallbackArguments;
use crate::workflow::retry::RetryArguments;
use crate::workflow::threshold::ThresholdArguments;
use crate::workflow::timeout::TimeoutArguments;
use crate::workflow::utils::WorkflowTransform;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, parenthesized};

pub struct WorkflowSpec(pub Punctuated<WorkflowPrimitive, Token![,]>);

pub enum WorkflowPrimitive {
    Retry(RetryArguments),
    Fallback(FallbackArguments),
    Delay(DelayArguments),
    Timeout(TimeoutArguments),
    Threshold(ThresholdArguments),
    Dependency(DependencyArguments),
    Condition(ConditionArguments),
}

impl WorkflowTransform for WorkflowPrimitive {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        match self {
            WorkflowPrimitive::Retry(res) => res.transform(toks),
            WorkflowPrimitive::Fallback(res) => res.transform(toks),
            WorkflowPrimitive::Delay(res) => res.transform(toks),
            WorkflowPrimitive::Timeout(res) => res.transform(toks),
            WorkflowPrimitive::Threshold(res) => res.transform(toks),
            WorkflowPrimitive::Dependency(res) => res.transform(toks),
            _ => todo!(),
            // WorkflowPrimitive::Condition(res) => res.transform(toks),
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        match self {
            WorkflowPrimitive::Retry(res) => res.get_type(toks),
            WorkflowPrimitive::Fallback(res) => res.get_type(toks),
            WorkflowPrimitive::Delay(res) => res.get_type(toks),
            WorkflowPrimitive::Timeout(res) => res.get_type(toks),
            WorkflowPrimitive::Threshold(res) => res.get_type(toks),
            WorkflowPrimitive::Dependency(res) => res.get_type(toks),
            _ => todo!(),
            // WorkflowPrimitive::Condition(res) => res.get_type(toks),
        }
    }
}

impl Parse for WorkflowPrimitive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>().map_err(|x| {
            syn::Error::new(
                x.span(),
                "Expected an identifier for the workflow primitive",
            )
        })?;

        let content;
        parenthesized!(content in input);

        match ident.to_string().as_str() {
            "retry" => Ok(Self::Retry(content.parse()?)),
            "delay" => Ok(Self::Delay(content.parse()?)),
            "timeout" => Ok(Self::Timeout(content.parse()?)),
            "fallback" => Ok(Self::Fallback(content.parse()?)),
            "threshold" => Ok(Self::Threshold(content.parse()?)),
            "dependency" => Ok(Self::Dependency(content.parse()?)),
            "condition" => Ok(Self::Condition(content.parse()?)),

            _ => Err(syn::Error::new_spanned(ident, "Unknown workflow primitive")),
        }
    }
}

impl Parse for WorkflowSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let primitives = Punctuated::parse_terminated(input)?;
        if primitives.is_empty() {
            return Err(input.error("Expected at least one workflow primitive"));
        }

        Ok(Self(primitives))
    }
}

pub fn workflow(_attrs: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "Workflow attribute is unsupported outside of Tasks and TaskFrames (via the respective macros)"
    ).to_compile_error().into()
}
