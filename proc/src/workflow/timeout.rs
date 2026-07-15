use crate::utils::TimeLiteral;
use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};

pub struct TimeoutArguments {
    duration: ValueSource<TimeLiteral>,
    on_timeout: Option<syn::Expr>,
}

impl Parse for TimeoutArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let duration = argument_parser.parse_required("duration")?;
        let on_timeout = argument_parser.parse_optional("on_timeout")?;
        Ok(TimeoutArguments { duration, on_timeout })
    }
}

impl WorkflowTransform for TimeoutArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let duration = &self.duration;
        let on_timeout = self.on_timeout
            .as_ref()
            .map(|x| quote! { .on_timeout(#x) });

        quote! {
            ::chronographer::task::frames::timeoutframe::TimeoutTaskFrame::builder()
                .frame(#toks)
                .duration(#duration)
                #on_timeout
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        quote! { ::chronographer::task::frames::timeoutframe::TimeoutTaskFrame::<#toks> }
    }
}
