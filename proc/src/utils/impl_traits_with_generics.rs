use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

pub(crate) fn derive_with_generics(name: &syn::Ident, fn_sig: &syn::Signature) -> syn::Result<(
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<Punctuated<proc_macro2::TokenStream, Comma>>
)> {
    let generics = &fn_sig.generics;
    let where_clause = &fn_sig.generics.where_clause;
    let mut phantom_data = None;
    let mut derives = quote! { #[derive(Default, Clone, Copy)] };
    let mut impl_end_name = quote! { #name };
    let mut normalized_type_params = None;

    if let Some(lt) = generics.lifetimes().next() {
        return Err(syn::Error::new(
            lt.span(),
            "Lifetimes are unsupported due to 'static lifetime limitations from async",
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
        derives = quote! {
            impl #generics Default for #impl_end_name {
                fn default() -> Self {
                    Self(std::marker::PhantomData)
                }
            }

            impl #generics Clone for #impl_end_name {
                fn clone(&self) -> Self {
                    Self(std::marker::PhantomData)
                }
            }

            impl #generics Copy for #impl_end_name {}
        };
    }

    Ok((derives, impl_end_name, phantom_data, normalized_type_params))
}