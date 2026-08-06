use proc_macro::TokenStream;
use std::collections::HashSet;
use darling::ast::{GenericParamExt, NestedMeta};
use darling::FromMeta;
use quote::{quote, ToTokens};
use syn::{ItemEnum, Token};
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::event::utils::{get_ident_from_generic, parse_individual_event, IndividualEventMacroArguments, Payload};

struct EventEnumMacroArguments(Option<Payload>);

impl Parse for EventEnumMacroArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self(None));
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "payload" {
            return Err(input.error("Expected \"payload\" as the only parameter but got something else"));
        }

        input.parse::<Token![=]>()?;

        let payload = input.parse::<Payload>()?.anonymize();
        if !input.is_empty() {
            return Err(input.error("Unexpected subsequent tokens found"))
        }

        Ok(Self(Some(payload)))
    }
}

pub fn parse_event_enum(attr: TokenStream, item: ItemEnum) -> syn::Result<TokenStream> {
    let theg_name = item.ident;
    let theg_generics = item.generics;
    let theg_variants = item.variants;
    let theg_vis = item.vis;
    let theg_attrs = item.attrs;

    let payload_arg = EventEnumMacroArguments::parse.parse2(attr.into())?.0;

    let mut lifetimes = theg_generics.lifetimes();
    let payload_lt = lifetimes.next()
        .cloned()
        .map(|lt| lt.lifetime)
        .unwrap_or(syn::Lifetime::new(
            "'a",
            proc_macro2::Span::call_site()
        ));

    if let Some(lt) = lifetimes.next() {
        return Err(syn::Error::new_spanned(
            lt,
            "Event cannot have more than one lifetime parameter (the payload)",
        ))
    }

    let sealed_name = syn::Ident::new(&format!("Sealed{theg_name}"), theg_name.span());
    let other_theg_generics = theg_generics.params.iter()
        .filter(|&x| x.as_lifetime_param().is_none())
        .cloned()
        .collect::<Punctuated<_, Comma>>();

    let theg_where_clause = theg_generics.where_clause.as_ref();
    let theg_ty_generics = other_theg_generics.iter()
        .map(|x| get_ident_from_generic(&x).to_token_stream())
        .collect::<Punctuated<_, Comma>>();

    let mut expanded_variants = Vec::new();
    let mut names_list = HashSet::new();
    for variant in theg_variants {
        let variant_attrs = &variant.attrs;
        let filtered_variant_attrs = variant_attrs
            .iter()
            .filter(|&attr| !attr.path().is_ident("event"))
            .cloned()
            .collect::<Vec<_>>();

        let event_attr = variant_attrs.iter()
            .find(|&attr| attr.path().is_ident("event"));

        let variant_name = &variant.ident;
        let variant_fields = &variant.fields;

        if variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(variant, "Cannot assign values to event variants"))
        } else if names_list.contains(&variant_name.to_string()) {
            return Err(syn::Error::new_spanned(variant, "Duplicate event name defined"))
        }

        let mut individual_args = IndividualEventMacroArguments::from_list(&[])?;
        if let Some(attr) = event_attr {
            let variant_metalist = NestedMeta::parse_meta_list(attr.meta.require_list()?.tokens.clone())?;
            individual_args = IndividualEventMacroArguments::from_list(&*variant_metalist)?
        }

        let expanded_defs = parse_individual_event(
            individual_args,
            &filtered_variant_attrs,
            &theg_vis,
            &variant_name,
            &theg_generics,
            &variant_fields
        )?;

        expanded_variants.push(quote! {
            #expanded_defs

            impl <#other_theg_generics> #sealed_name for #variant_name <#theg_ty_generics> #theg_where_clause {}
            impl <#other_theg_generics> #theg_name <#theg_ty_generics> for #variant_name <#theg_ty_generics> #theg_where_clause {}
        });

        names_list.insert(variant_name.to_string());
    }

    let taskhook_event = if let Some(payload) = payload_arg {
        quote! { for<#payload_lt> ::chronographer::task::hooks::TaskHookEvent<Payload<#payload_lt> = #payload> }
    } else { quote! { ::chronographer::task::hooks::TaskHookEvent }};

    Ok(quote! {
        trait #sealed_name {}

        #(#theg_attrs)*
        #theg_vis trait #theg_name <#other_theg_generics>: #sealed_name + #taskhook_event #theg_where_clause {}

        #(#expanded_variants)*
    }.into())
}