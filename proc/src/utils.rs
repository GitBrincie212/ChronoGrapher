use quote::quote;
use syn::Attribute;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

pub mod time_literal;

pub(crate) const LIFETIME_UNSUPPORTED_ERR: &'static str = "Lifetimes are unsupported due to 'static lifetime limitations from async";

pub fn extract_docs(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    attrs.iter()
        .filter_map(|a| {
            if !a.path().is_ident("doc") {
                return None;
            }

            let syn::Meta::NameValue(nv) = &a.meta else { return None; };
            let syn::Expr::Lit(expr_lit) = &nv.value else { return None; };
            let syn::Lit::Str(lit) = &expr_lit.lit else { return None; };

            let string = lit.value();
            Some(quote! { #[doc = #string] })
        })
        .collect()
}

pub fn handle_generics_phantom_data(name: &syn::Ident, fn_sig: &syn::Signature) -> syn::Result<(
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<Punctuated<proc_macro2::TokenStream, Comma>>
)> {
    let generics = &fn_sig.generics;
    let where_clause = &fn_sig.generics.where_clause;
    let mut phantom_data = None;
    let mut impl_end_name = quote! { #name };
    let mut normalized_type_params = None;

    if let Some(lt) = generics.lifetimes().next() {
        return Err(syn::Error::new(
            lt.span(),
            LIFETIME_UNSUPPORTED_ERR
        ));
    }

    if !generics.params.is_empty() {
        let phantom_type_params = generics.type_params()
            .map(|x| {
                let type_param = &x.ident;
                quote! { #type_param }
            })
            .collect::<Punctuated<_, Comma>>();

        phantom_data = Some(quote! { ( std::marker::PhantomData <( #phantom_type_params )> ) });

        let mut temp = phantom_type_params.clone();
        temp.extend(
            generics.const_params().map(|x| {
                let type_param = &x.ident;
                quote! { #type_param }
            })
        );

        normalized_type_params = Some(temp);

        impl_end_name = quote! { #name<#normalized_type_params> #where_clause };
    }

    Ok((impl_end_name, phantom_data, normalized_type_params))
}