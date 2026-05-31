use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use crate::workflow::utils::{ArgumentParser, WorkflowTransform};

pub struct FallbackArguments(Vec<syn::Ident>);

impl Parse for FallbackArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let fallbacks = argument_parser.parse_remaining::<syn::Ident>()?;
        Ok(FallbackArguments(fallbacks))
    }
}

impl WorkflowTransform for FallbackArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for ident in self.0.iter() {
            curr = quote! {
                chronographer::prelude::FallbackTaskFrame::new(
                    #curr,
                    #ident
                )
            }
        }

        curr
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for ident in self.0.iter() {
            curr = quote! { chronographer::prelude::FallbackTaskFrame<#curr, #ident> };
        }
        
        curr
    }
}