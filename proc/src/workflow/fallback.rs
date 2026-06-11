use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use crate::utils::TaskFrameConstructor;

pub struct FallbackArguments(Vec<TaskFrameConstructor>);

impl Parse for FallbackArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let fallbacks = argument_parser.parse_remaining::<TaskFrameConstructor>()?;
        Ok(FallbackArguments(fallbacks))
    }
}

impl WorkflowTransform for FallbackArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for expr in self.0.iter() {
            let output = &expr.to_token_construction();
            curr = quote! {
                ::chronographer::task::frames::fallbackframe::FallbackTaskFrame::new(
                    #curr,
                    #output
                )
            }
        }

        curr
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for expr in self.0.iter() {
            let output = &expr.to_token_type();
            curr = quote! {
                ::chronographer::task::frames::fallbackframe::FallbackTaskFrame::<
                    #curr,
                    #output
                >
            }
        }

        curr
    }
}
