use std::collections::{HashMap, HashSet};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::__private::TokenStream2;
use syn::parenthesized;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub enum HookAnnotationStatement {
    InstanceCreation(syn::Ident, syn::Expr),
    ManualBasedListen(syn::Ident, syn::Expr),
    FunctionBasedListen(syn::Expr),
    AutoBasedListen(syn::Expr),
}

impl Parse for HookAnnotationStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if input.peek(syn::Token![=]) {
            let _ = input.parse::<syn::Token![=]>()?;
            let expr = input.parse::<syn::Expr>()?;
            return Ok(HookAnnotationStatement::InstanceCreation(ident, expr));
        }

        if input.peek(syn::Token![:]) {
            let _ = input.parse::<syn::Token![:]>()?;
            let expr = input.parse::<syn::Expr>()?;
            return Ok(HookAnnotationStatement::ManualBasedListen(ident, expr));
        }

        let is_auto = ident.to_string() == "auto";
        let is_custom = ident.to_string() == "fn";
        if !is_auto && !is_custom {
            return Err(input.error("Expected either \"auto\" or \"fn\" but got something else\""))
        }

        let content;
        parenthesized!(content in input);

        if is_auto {
            return Ok(HookAnnotationStatement::AutoBasedListen(content.parse()?));
        }

        Ok(HookAnnotationStatement::FunctionBasedListen(content.parse()?))
    }
}

impl HookAnnotationStatement {
    pub fn to_tokens(&self, tokens: &mut TokenStream2, task: syn::Ident) {
        let expanded = match self {
            HookAnnotationStatement::InstanceCreation(ident, expr) => quote! {let #ident = std::sync::Arc::from(#expr); },
            HookAnnotationStatement::ManualBasedListen(ident, expr) => quote! {#task.attach_hook::<#ident>(#expr).await; },
            HookAnnotationStatement::FunctionBasedListen(expr) => quote! { #expr(&#task).await; },
            HookAnnotationStatement::AutoBasedListen(expr) => quote! { std::sync::Arc::from(#expr).auto_attach(&#task).await; },
        };

        tokens.append_all(expanded);
    }
}

pub struct HookAnnotationArguments(Punctuated<HookAnnotationStatement, Comma>);

impl Parse for HookAnnotationArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let values = Punctuated::parse_separated_nonempty_with(input, |x| x.parse())?;
        let mut instances = HashMap::new();
        let mut auto_instances = HashSet::new();
        for val in values.iter() {
            match val {
                HookAnnotationStatement::InstanceCreation(name, _) => {
                    if instances.contains_key(name) {
                        return Err(input.error(format!("Duplicate instance name \"{}\"", name)));
                    }

                    instances.insert(name.clone(), ());
                }

                HookAnnotationStatement::ManualBasedListen(_, value) => {
                    let syn::Expr::Path(pt) = value else {
                        continue;
                    };

                    if pt.path.segments.len() == 1 && !instances.contains_key(&pt.path.segments[0].ident) {
                        return Err(input.error(format!("Invalid manual attachment for non-existent instance \"{}\"", pt.path.segments[0].ident)));
                    };
                }

                HookAnnotationStatement::AutoBasedListen(value) => {
                    let syn::Expr::Path(pt) = value else {
                        continue;
                    };

                    if pt.path.segments.len() == 1 && !instances.contains_key(&pt.path.segments[0].ident) {
                        return Err(input.error(format!("Invalid manual attachment for non-existent instance \"{}\"", pt.path.segments[0].ident)));
                    };

                    let name = &pt.path.segments[0].ident;
                    if auto_instances.contains(name) {
                        return Err(input.error(format!("Duplicate auto-attachment for identical instance \"{name}\"")));
                    }

                    auto_instances.insert(name.clone());
                }

                HookAnnotationStatement::FunctionBasedListen(_) => {}
            }
        }

        Ok(HookAnnotationArguments(values))
    }
}

impl ToTokens for HookAnnotationArguments {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for value in &self.0 {
            value.to_tokens(tokens, syn::Ident::new("task", proc_macro2::Span::call_site()));
        }
    }
}