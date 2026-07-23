use darling::ast::GenericParamExt;
use darling::FromMeta;
use darling::util::Flag;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{GenericParam, Lifetime, LifetimeParam, Token};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(FromMeta, Debug, Clone)]
pub struct IndividualEventMacroArguments {
    pub inline: Flag,
    pub payload_name_override: Option<syn::Ident>,
}

pub struct PayloadField {
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

pub enum Payload {
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

pub fn parse_individual_event(
    args: IndividualEventMacroArguments,
    event_attrs: &[syn::Attribute],
    event_vis: &syn::Visibility,
    event_name: &syn::Ident,
    event_generics: &syn::Generics,
    event_fields: &syn::Fields,
) -> syn::Result<TokenStream2> {
    if args.inline.is_present() && args.payload_name_override.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "Cannot override payload name while inlining it"))
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

        syn::Fields::Named(fields) if !fields.named.is_empty() => {
            payload_ty.extend(fields.named.iter().map(|field| {
                let ty = &field.ty;
                quote! { #ty }
            }))
        },

        syn::Fields::Unnamed(fields) if args.inline.is_present() => {
            return Err(syn::Error::new_spanned(fields, "Cannot use the inline attribute for tuple-based payloads"))
        }

        syn::Fields::Unnamed(fields) if !fields.unnamed.is_empty() => {
            payload_ty.extend(fields.unnamed.iter().map(|x| x.to_token_stream()))
        },

        _ => {}
    }

    let default_trait_impl = if other_generics.is_empty() {
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

    let other_attributes = event_attrs.iter().find(|&attr| {
        let syn::Meta::List(list) = &attr.meta else {
            return true;
        };

        return !(list.path.segments.len() == 1 && list.path.segments[0].ident.to_string() == "derive");
    });

    Ok(quote! {
        #default_trait_impl
        #other_attributes
        #event_vis struct #event_name<#other_generics_impl> #event_phantom_data;

        #payload_struct_ty

        impl <#other_generics_impl> ::chronographer::task::hooks::TaskHookEvent for #event_name <#other_generics> {
            type Payload<#payload_lt> = (#payload_ty) where Self: #payload_lt;
        }
    })
}