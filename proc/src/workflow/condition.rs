use crate::utils::TaskFrameConstructor;
use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};

pub struct ConditionArguments {
    predicate: syn::Ident,
    secondary: Option<TaskFrameConstructor>,
}

impl Parse for ConditionArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let predicate = argument_parser.parse_required("predicate")?;
        let secondary = argument_parser.parse_optional("backup")?;
        Ok(ConditionArguments {
            predicate,
            secondary,
        })
    }
}

impl WorkflowTransform for ConditionArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let predicate = &self.predicate;
        let secondary = self.secondary.as_ref().map(|secondary| {
            let output = secondary.to_token_construction();
            quote! { .fallback(#output) }
        });

        quote! {
            ::chronographer::task::frames::conditionframe::ConditionalTaskFrame::builder()
                .predicate(#predicate)
                .frame(#toks)
                #secondary
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        self.secondary
            .as_ref()
            .map(|secondary| {
                let output = secondary.to_token_type();
                quote! { #output }
            })
            .unwrap_or_else(|| {
                quote! {
                    ::chronographer::task::frames::conditionframe::ConditionalTaskFrame::<
                        #toks,
                        ::chronographer::task::frames::noopframe::NoOperationTaskFrame::<
                            <#toks as ::chronographer::task::frames::TaskFrame>::Error,
                            ()
                        >
                    >
                }
            })
    }
}
