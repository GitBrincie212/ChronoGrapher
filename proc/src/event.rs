pub mod event_struct;

use proc_macro::TokenStream;
use syn::parse_macro_input;

pub fn event(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);

    match item {
        syn::Item::Mod(mod_item) => {
            todo!()
        }

        syn::Item::Trait(fn_item) => {
            todo!()
        }

        syn::Item::Enum(fn_item) => {
            todo!()
        }

        syn::Item::Struct(struct_item) => {
            event_struct::parse_event_struct(attrs, struct_item)
                .unwrap_or_else(|err| err.into_compile_error().into())
        }

        _ => syn::Error::new(
            proc_macro2::Span::call_site(),
            "Macro cannot be used on this items, apply it to only modules, traits, enums or structs"
        ).into_compile_error().into(),
    }
}