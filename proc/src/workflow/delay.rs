use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use crate::utils::TimeLiteral;
use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};

pub struct DelayArguments(ValueSource<TimeLiteral>);

impl Parse for DelayArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let delay = argument_parser.parse_required("delay")?;
        Ok(DelayArguments(delay))
    }
}

impl WorkflowTransform for DelayArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let value = &self.0;
        
        let method_name = match &value {
            ValueSource::Function(_) => quote! { new_with },
            _ => quote! { new }
        };
        
        quote! { chronographer::task::frames::delayframe::DelayTaskFrame::#method_name( #toks, #value )}
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        quote! { chronographer::task::frames::delayframe::DelayTaskFrame<#toks> }
    }
}