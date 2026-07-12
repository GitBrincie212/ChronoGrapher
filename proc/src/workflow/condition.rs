use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parenthesized;
use syn::parse::{Parse, ParseStream};
use crate::utils::TaskFrameConstructor;

pub enum ConditionReturnBehavior {
    Error,
    Skip,
    Custom(syn::Expr)
}

impl Parse for ConditionReturnBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "error" => Ok(Self::Error),
            "skip" => Ok(Self::Skip),
            "custom" => {
                let content;
                parenthesized!(content in input);

                let expr = content.parse()?;
                if matches!(expr, syn::Expr::Macro(_) | syn::Expr::Path(_) | syn::Expr::Closure(_)) {
                    return Ok(Self::Custom(expr))
                }

                Err(input.error("Expected a macro, function or closure expression but got something else"))
            }
            _ => Err(input.error("Unknown condition return behaviour")),
        }
    }
}

impl ToTokens for ConditionReturnBehavior {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match &self {
            ConditionReturnBehavior::Error => quote! { .on_false_error() },
            ConditionReturnBehavior::Skip => quote! { .on_false_skip() },
            &ConditionReturnBehavior::Custom(value) => quote! { .on_false_custom(#value) },
        };

        tokens.append_all(expanded)
    }
}

pub struct ConditionArguments {
    predicate: syn::Ident,
    secondary: Option<TaskFrameConstructor>,
    on_false: Option<ConditionReturnBehavior>,
}

impl Parse for ConditionArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let predicate = argument_parser.parse_required("predicate")?;
        let secondary = argument_parser.parse_optional("secondary")?;
        let on_false = argument_parser.parse_optional("on_false")?;
        Ok(ConditionArguments {
            predicate,
            secondary,
            on_false,
        })
    }
}

impl WorkflowTransform for ConditionArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let predicate = &self.predicate;
        let on_false = &self.on_false;
        let secondary = self.secondary.as_ref()
            .map(|secondary| {
                let output = secondary.to_token_construction();
                quote! { .fallback(#output) }
            });

        let builder_method = if secondary.is_some() {
            quote! { fallback_builder }
        } else {
            quote! { builder }
        };

        quote! {
            ::chronographer::task::frames::conditionframe::ConditionalTaskFrame:: #builder_method()
                .predicate(#predicate)
                .frame(#toks)
                #secondary
                #on_false
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        self.secondary.as_ref()
            .map(|secondary| {
                let output = secondary.to_token_type();
                quote! { #output }
            })
            .unwrap_or_else(|| quote! {
                ::chronographer::task::frames::conditionframe::ConditionalTaskFrame::<
                    #toks,
                    ::chronographer::task::frames::noopframe::NoOperationTaskFrame::<
                        <#toks as ::chronographer::task::frames::TaskFrame>::Error,
                        ()
                    >
                >
            })
    }
}
