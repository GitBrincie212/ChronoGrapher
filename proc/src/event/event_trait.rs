use proc_macro::TokenStream;
use darling::ast::GenericParamExt;
use quote::quote;
use syn::{bracketed, ItemTrait};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::event::utils::{get_ident_from_generic, Payload};

struct EventTraitMacroArguments {
    blanket: Option<Punctuated<syn::Type, Comma>>,
    payload: Option<Payload>,
}

impl Parse for EventTraitMacroArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut blanket = None;
        let mut payload = None;

        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            if ident.to_string() != "payload" && ident.to_string() != "blanket" {
                return Err(input.error("Expected either \"payload\" or \"blanket\" as parameters but got something else"));
            }

            let _ = input.parse::<syn::Token![=]>()?;

            if ident.to_string() == "payload" {
                payload = Some(input.parse::<Payload>()?.anonymize())
            } else {
                blanket = if input.peek(syn::token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    Some(Punctuated::<_, Comma>::parse_terminated(&content)?)
                } else {
                    let mut values = Punctuated::new();
                    values.push(input.parse()?);
                    Some(values)
                };
            }

            let comma = input.parse::<syn::Token![,]>();
            if !input.is_empty() { comma?; }
        }

        Ok(Self {
            blanket,
            payload
        })
    }
}

pub fn parse_event_trait(attr: TokenStream, item: ItemTrait) -> syn::Result<TokenStream> {
    let theg_name = item.ident;
    let theg_generics = item.generics;
    let theg_bounds = item.supertraits;
    let theg_vis = item.vis;
    let theg_attrs = item.attrs;
    let theg_unsafety = item.unsafety;

    if !item.items.is_empty() {
        return Err(syn::Error::new_spanned(&item.items[0], "Trait items are not allowed when defining a THEG"))
    }

    let args = EventTraitMacroArguments::parse.parse2(attr.clone().into())?;

    let mut lifetimes = theg_generics.lifetimes();
    let first_lt = lifetimes.next();
    let payload_lt = first_lt
        .cloned()
        .map(|lt| lt.lifetime)
        .unwrap_or(syn::Lifetime::new(
            "'a",
            proc_macro2::Span::call_site()
        ));

    let other_params_impl = theg_generics.params.iter()
        .filter(|x| x.as_lifetime_param().is_none())
        .collect::<Punctuated<_, Comma>>();

    let other_params = other_params_impl.iter()
        .map(get_ident_from_generic)
        .collect::<Punctuated<_, Comma>>();

    if let Some(lt) = lifetimes.next() {
        return Err(syn::Error::new_spanned(
            lt,
            "Event cannot have more than one lifetime parameter (the payload)",
        ))
    }

    let taskhook_event = if let Some(payload) = args.payload {
        quote! { for<#payload_lt> ::chronographer::task::hooks::TaskHookEvent<Payload<#payload_lt> = #payload> }
    } else { quote! { ::chronographer::task::hooks::TaskHookEvent } };

    let blanket_arg = args.blanket.unwrap_or(Punctuated::new());
    let blanket_impls = blanket_arg.iter()
        .map(|blanket| quote! { #theg_unsafety impl<#other_params_impl> #theg_name<#other_params> for #blanket {} })
        .collect::<Vec<_>>();

    Ok(quote! {
        #theg_unsafety #theg_vis trait #theg_name <#other_params_impl>: #taskhook_event + #theg_bounds  {}

        #(#blanket_impls)*
    }.into())
}