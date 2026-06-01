use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use crate::utils::TimeLiteral;
use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};

pub struct TimeoutArguments(ValueSource<TimeLiteral>);

impl Parse for TimeoutArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let duration = argument_parser.parse_required("duration")?;
        Ok(TimeoutArguments(duration))
    }
}

impl WorkflowTransform for TimeoutArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let value = &self.0;

        let method_name = match &value {
            ValueSource::Function(_) => quote! { new_with },
            _ => quote! { new }
        };

        quote! { chronographer::task::frames::timeoutframe::TimeoutTaskFrame::#method_name( #toks, #value )}
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        quote! { chronographer::task::frames::timeoutframe::TimeoutTaskFrame<#toks> }
    }
}