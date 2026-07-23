pub mod event_struct;
pub mod event_enum;
pub mod utils;
pub mod event_trait;

use proc_macro::TokenStream;
use syn::parse_macro_input;

pub fn event(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);

    match item {
        // TODO: Add support for "mod" blocks acting as open-form THEGs in the future

        syn::Item::Trait(trait_item) => event_trait::parse_event_trait(attrs, trait_item)
            .unwrap_or_else(|err| err.into_compile_error().into()),

        syn::Item::Enum(enum_item) => event_enum::parse_event_enum(attrs, enum_item)
            .unwrap_or_else(|err| err.into_compile_error().into()),

        syn::Item::Struct(struct_item) => event_struct::parse_event_struct(attrs, struct_item)
            .unwrap_or_else(|err| err.into_compile_error().into()),

        _ => syn::Error::new(
            proc_macro2::Span::call_site(),
            "Macro cannot be used on this items, apply it to only modules, traits, enums or structs"
        ).into_compile_error().into(),
    }
}