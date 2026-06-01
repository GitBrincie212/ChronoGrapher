use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Token;

pub struct FallbackArguments(Vec<TaskFrameExpr>);

pub enum TaskFrameExpr {
    Single(syn::Type),
    Workflow(syn::Type),
}

impl TaskFrameExpr {
    fn get_type(&self) -> TokenStream2 {
        match self {
            TaskFrameExpr::Single(ident) => quote! { #ident },
            TaskFrameExpr::Workflow(ident) => quote! { <#ident as ::chronographer::task::frames::TaskFrame>::Workflow },
        }
    }

    fn get_constructor(&self) -> TokenStream2 {
        match self {
            TaskFrameExpr::Single(ident) => quote! { #ident::single() },
            TaskFrameExpr::Workflow(ident) => quote! { #ident::workflow() },
        }
    }
}

impl Parse for TaskFrameExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            let _: Token![@] = input.parse()?;
            let identifier = input.parse()?;
            return Ok(Self::Single(identifier))
        }

        let identifier = input.parse()?;
        Ok(Self::Workflow(identifier))
    }
}

impl Parse for FallbackArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let fallbacks = argument_parser.parse_remaining::<TaskFrameExpr>()?;
        Ok(FallbackArguments(fallbacks))
    }
}

impl WorkflowTransform for FallbackArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for expr in self.0.iter() {
            let constructor = expr.get_constructor();
            curr = quote! {
                ::chronographer::task::frames::fallbackframe::FallbackTaskFrame::new(
                    #curr,
                    #constructor
                )
            }
        }

        curr
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for expr in self.0.iter() {
            let ty = expr.get_type();
            curr = quote! {
                ::chronographer::task::frames::fallbackframe::FallbackTaskFrame::<
                    #curr,
                    #ty
                >
            }
        }

        curr
    }
}
