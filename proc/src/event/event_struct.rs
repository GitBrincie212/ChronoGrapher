use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use darling::ast::{GenericParamExt, NestedMeta};
use darling::FromMeta;
use darling::util::Flag;
use quote::{quote, ToTokens};
use syn::{GenericParam, ItemStruct, Lifetime, LifetimeParam};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(FromMeta, Debug, Clone)]
pub struct EventStructMacroArguments {
    inline: Flag,
    payload_name_override: Option<syn::Ident>,
}

pub fn parse_event_struct(attr: TokenStream, item: ItemStruct) -> syn::Result<TokenStream> {
    let event_name = item.ident;
    let event_generics = item.generics;
    let event_fields = item.fields;
    let event_vis = item.vis;
    let event_attrs = item.attrs;

    let attr_args: Vec<NestedMeta> = NestedMeta::parse_meta_list(attr.into())?;
    let args = EventStructMacroArguments::from_list(&attr_args)?;

    if args.inline.is_present() && args.payload_name_override.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "Cannot override payload name when inlining it"))
    }

    let mut lifetimes = event_generics.lifetimes();
    let first_lt = lifetimes.next();
    let payload_lt = first_lt.cloned().unwrap_or_else(|| {
        LifetimeParam::new(Lifetime::new("'a", proc_macro2::Span::call_site()))
    });

    if let Some(lt) = lifetimes.next() {
        return Err(syn::Error::new_spanned(lt, "Event cannot have more than one lifetime parameter (the payload)"));
    }

    let other_generics_impl = event_generics.params.iter()
        .filter(|param| param.as_lifetime_param().is_none())
        .collect::<Punctuated<_, Comma>>();

    let other_generics = other_generics_impl.iter()
        .map(|&param| match param {
            GenericParam::Type(ty) => ty.ident.clone(),
            GenericParam::Const(ct) => ct.ident.clone(),
            _ => unreachable!()
        })
        .collect::<Punctuated<_, Comma>>();

    let event_phantom_data = if !other_generics.is_empty() {
        Some(quote! { (std::marker::PhantomData<(#other_generics)>) })
    } else { None };

    let mut payload_struct_ty: Option<TokenStream2> = None;
    let mut payload_ty: Punctuated<_, Comma> = Punctuated::new();
    match event_fields {
        syn::Fields::Named(fields) if !fields.named.is_empty() && !args.inline.is_present() => {
            let payload_name = args.payload_name_override.unwrap_or_else(|| {
                syn::Ident::new(&format!("{}Payload", event_name), proc_macro2::Span::call_site())
            });

            let (_, ty_generics, where_clause) = event_generics.split_for_impl();
            payload_ty.push(quote! { #payload_name #ty_generics #where_clause });
            let derives = event_attrs.iter().find(|&attr| {
                let syn::Meta::List(list) = &attr.meta else {
                    return false;
                };

                return list.path.segments.len() == 1
                    && list.path.segments[0].ident.to_string() == "derive";
            });

            payload_struct_ty = Some(quote! {
                #derives
                #event_vis struct #payload_ty #fields
            })
        },

        syn::Fields::Named(fields) if !fields.named.is_empty() && args.inline.is_present() => {
            payload_ty.extend(fields.named.iter().map(|field| {
                let ty = &field.ty;
                quote! { #ty }
            }))
        },

        syn::Fields::Named(fields) if !fields.named.is_empty()
            && args.inline.is_present() => {
            payload_ty.extend(fields.named.iter().map(|x| x.to_token_stream()))
        },

        syn::Fields::Unnamed(fields) if !fields.unnamed.is_empty() => {
            payload_ty.extend(fields.unnamed.iter().map(|x| x.to_token_stream()))
        },

        _ => {}
    }

    let derive_traits = if other_generics.is_empty() {
        quote! { #[derive(Default)] }
    } else {
        quote! {
            impl <#other_generics_impl> Default for #event_name <#other_generics> {
                fn default() -> Self {
                    Self(Default::default())
                }
            }
        }
    };

    Ok(quote! {
        #derive_traits
        #event_vis struct #event_name<#other_generics_impl> #event_phantom_data;

        #payload_struct_ty

        impl <#other_generics_impl> ::chronographer::task::hooks::TaskHookEvent for #event_name <#other_generics> {
            type Payload<#payload_lt> = (#payload_ty) where Self: #payload_lt;
        }
    }.into())
}