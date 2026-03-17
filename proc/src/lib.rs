mod every;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro]
pub fn every(input: TokenStream) -> TokenStream {
    every::every(input)
}

#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn greet() -> String {
                my_library::hello(stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}