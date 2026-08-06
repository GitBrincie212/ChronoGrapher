use crate::utils::TimeLiteral;
use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parenthesized;
use syn::parse::{Parse, ParseBuffer, ParseStream};

enum OnTimeoutVariants {
    Constant(syn::Expr),
    Dynamic(syn::Expr),
}

fn try_dynamic_path(fork: ParseBuffer) -> syn::Result<OnTimeoutVariants> {
    let ident = fork.parse::<syn::Ident>()?;
    let content;
    parenthesized!(content in fork);

    if ident != "dynamic" {
        return Err(syn::Error::new_spanned(
            ident,
            "Expected 'dynamic' but got something else",
        ));
    }

    let expr = content.parse::<syn::Expr>()?;
    match expr {
        syn::Expr::Closure(_) => Ok(OnTimeoutVariants::Dynamic(expr)),
        syn::Expr::Path(_) => Ok(OnTimeoutVariants::Dynamic(expr)),
        syn::Expr::Macro(_) => Ok(OnTimeoutVariants::Dynamic(expr)),
        _ => Err(syn::Error::new_spanned(
            expr,
            "Expected a macro, closure or function identifier",
        )),
    }
}

impl Parse for OnTimeoutVariants {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(value) = try_dynamic_path(input.fork()) {
            return Ok(value);
        }

        let expr = input.parse()?;
        match expr {
            syn::Expr::Path(_) => Ok(OnTimeoutVariants::Constant(expr)),
            syn::Expr::Macro(_) => Ok(OnTimeoutVariants::Constant(expr)),
            _ => Err(syn::Error::new_spanned(
                expr,
                "Expected an error identifier or macro",
            )),
        }
    }
}

pub struct TimeoutArguments {
    duration: ValueSource<TimeLiteral>,
    on_timeout: Option<OnTimeoutVariants>,
}

impl Parse for TimeoutArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let duration = argument_parser.parse_required("duration")?;
        let on_timeout = argument_parser.parse_optional("on_timeout")?;
        Ok(TimeoutArguments {
            duration,
            on_timeout,
        })
    }
}

impl WorkflowTransform for TimeoutArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let duration = &self.duration;
        let on_timeout = self.on_timeout.as_ref().map(|value| match value {
            OnTimeoutVariants::Constant(expr) => quote! { .on_timeout(#expr) },
            OnTimeoutVariants::Dynamic(expr) => quote! { .on_timeout_fn(#expr) },
        });

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
