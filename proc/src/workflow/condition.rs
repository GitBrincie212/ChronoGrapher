use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use crate::workflow::utils::{ArgumentParser, WorkflowTransform};

// TODO: Work on custom-based error behaviours
pub enum ConditionReturnBehavior {
    Error,
    Success,
    // Custom(syn::Expr)
}

impl Parse for ConditionReturnBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "error" => Ok(Self::Error),
            "success" => Ok(Self::Success),
            /*
            "custom" => {
                let content;
                parenthesized!(content in input);

                Ok(Self::Custom(content.parse()?))
            }
             */

            _ => Err(input.error("Unknown condition return behaviour"))
        }
    }
}

impl ToTokens for ConditionReturnBehavior {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            ConditionReturnBehavior::Error => quote! { true },
            ConditionReturnBehavior::Success => quote! { false }
        };

        tokens.append_all(expanded)
    }
}


// TODO: Fix some errors regarding impl TaskFrame when generating the macro
pub struct ConditionArguments {
    predicate: syn::Ident,
    secondary: Option<syn::Ident>,
    on_false: Option<ConditionReturnBehavior>,
}

impl Parse for ConditionArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let predicate = argument_parser.parse_required("predicate")?;
        let secondary = argument_parser.parse_optional("secondary")?;
        let on_false = argument_parser.parse_optional("on_false")?;
        Ok(ConditionArguments { predicate, secondary, on_false })
    }
}

impl WorkflowTransform for ConditionArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let predicate = &self.predicate;
        let secondary = self.secondary.as_ref()
            .map(|x| quote! { .fallback(#x) });

        let on_false = self.on_false.as_ref()
            .map(|x| quote! { .error_on_false(#x) });

        let builder_method = if secondary.is_some() {
            quote! { fallback_builder }
        } else { quote! { builder }};

        quote! {
            chronographer::task::frames::conditionframe::ConditionalTaskFrame:: #builder_method()
                .predicate(#predicate)
                .frame(#toks)
                #secondary
                #on_false
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        let secondary = self.secondary.as_ref()
            .map(|x| quote! { #x })
            .unwrap_or_else(|| quote! { chronographer::task::frames::noopframe::NoOperationTaskFrame<_, ()> });

        quote! {
            chronographer::task::frames::conditionframe::ConditionalTaskFrame<#toks, #secondary>
        }
    }
}