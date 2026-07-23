use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use darling::ast::{GenericParamExt, NestedMeta};
use darling::FromMeta;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{GenericParam, ItemEnum, Token};
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::event::utils::{parse_individual_event, IndividualEventMacroArguments};

struct PayloadField {
    ident: syn::Ident,
    ty: syn::Type,
}

impl Parse for PayloadField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _ = input.parse::<Token![:]>()?;
        let ty = input.parse::<syn::Type>()?;

        Ok(Self { ident, ty })
    }
}

impl ToTokens for PayloadField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.ident;
        let ty = &self.ty;

        tokens.append_all(quote! { #ident: #ty });
    }
}

enum Payload {
    Type(syn::Type),
    UnnamedFields(Punctuated<syn::Type, Comma>),
    NamedFields(Punctuated<PayloadField, Comma>),
}

impl ToTokens for Payload {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            Payload::Type(ty) => ty.to_token_stream(),
            Payload::UnnamedFields(fields) => quote! { ( #fields ) },
            Payload::NamedFields(fields) => quote! { { #fields } },
        };

        tokens.append_all(expanded)
    }
}

struct EventEnumMacroArguments {
    payload: Option<Payload>,
    payload_name_override: Option<syn::Ident>,
}

impl Parse for EventEnumMacroArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut payload = None;
        let mut payload_name_override = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident != "payload" && ident != "payload_name_override" {
                return Err(input.error("Expected either \"payload\" or \"payload_name_override\" as parameters but got something else"));
            }

            if ident == "payload_name_override" {
                if payload_name_override.is_some() {
                    return Err(input.error("Duplicate definition of \"payload_name_override\" parameter"));
                }

                input.parse::<Token![=]>()?;
                payload_name_override = Some(input.parse()?);
                continue;
            }

            if payload.is_some() {
                return Err(input.error("Duplicate definition of \"payload\" parameter"));
            }

            input.parse::<Token![=]>()?;
            if input.peek(syn::token::Brace) {
                let content;
                syn::braced!(content in input);
                let fields = Punctuated::<PayloadField, Comma>::parse_terminated(&content)?;
                payload = Some(Payload::NamedFields(fields));
                continue;
            } else if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let fields = Punctuated::<syn::Type, Comma>::parse_terminated(&content)?;
                payload = Some(Payload::UnnamedFields(fields));
                continue;
            }

            let ty: syn::Type = input.parse()?;
            payload = Some(Payload::Type(ty));

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(EventEnumMacroArguments {
            payload,
            payload_name_override,
        })
    }
}

pub fn parse_event_enum(attr: TokenStream, item: ItemEnum) -> syn::Result<TokenStream> {
    let theg_name = item.ident;
    let theg_generics = item.generics;
    let theg_variants = item.variants;
    let theg_vis = item.vis;
    let theg_attrs = item.attrs;

    let args = EventEnumMacroArguments::parse.parse2(attr.into())?;

    if args.payload.is_none() && args.payload_name_override.is_some() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Cannot override common payload name while not enabling it",
        ))
    }

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
        .map(|x| match x {
            GenericParam::Type(ty) => {
                let ident = ty.ident.clone();
                quote! { #ident }
            },

            GenericParam::Const(ct) => {
                let ident = ct.ident.clone();
                quote! { #ident }
            },

            _ => unreachable!()
        })
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

    let taskhook_event = if let Some(payload) = args.payload {
        quote! { for<#payload_lt> ::chronographer::task::hooks::TaskHookEvent<Payload<#payload_lt> = #payload> }
    } else { quote! { ::chronographer::task::hooks::TaskHookEvent }};

    Ok(quote! {
        trait #sealed_name {}

        #(#theg_attrs)*
        #theg_vis trait #theg_name <#other_theg_generics>: #sealed_name + #taskhook_event #theg_where_clause {}

        #(#expanded_variants)*
    }.into())
}