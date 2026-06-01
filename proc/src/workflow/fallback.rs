use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};

pub struct FallbackArguments(Vec<syn::Expr>);

impl Parse for FallbackArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let fallbacks = argument_parser.parse_remaining::<syn::Expr>()?;
        Ok(FallbackArguments(fallbacks))
    }
}

impl WorkflowTransform for FallbackArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for expr in self.0.iter() {
            curr = quote! {
                chronographer::task::frames::fallbackframe::FallbackTaskFrame::new(
                    #curr,
                    #expr
                )
            }
        }

        curr
    }
}
