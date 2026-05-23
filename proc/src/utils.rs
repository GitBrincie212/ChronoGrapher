use quote::quote;
use syn::Attribute;

pub mod time_literal;
pub mod impl_traits_with_generics;

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