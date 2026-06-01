use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use crate::workflow::utils::{ArgumentParser, WorkflowTransform};

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

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        let mut curr = quote! { #toks };
        for _ in self.0.iter() {
            curr = quote! {
                chronographer::task::frames::fallbackframe::FallbackTaskFrame<#curr, impl chronographer::task::frames::TaskFrame<
                    Error = <#curr as chronographer::task::frames::TaskFrame>::Error,
                    Args = <#curr as chronographer::task::frames::TaskFrame>::Args
                >>
            };
        }

        curr
    }
}