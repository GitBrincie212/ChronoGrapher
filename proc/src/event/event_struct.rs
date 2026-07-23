use proc_macro::TokenStream;
use darling::ast::NestedMeta;
use darling::FromMeta;
use syn::ItemStruct;
use crate::event::utils::{parse_individual_event, IndividualEventMacroArguments};

pub fn parse_event_struct(attr: TokenStream, item: ItemStruct) -> syn::Result<TokenStream> {
    let event_name = item.ident;
    let event_generics = item.generics;
    let event_fields = item.fields;
    let event_vis = item.vis;
    let event_attrs = item.attrs;

    let attr_args: Vec<NestedMeta> = NestedMeta::parse_meta_list(attr.into())?;
    let args = IndividualEventMacroArguments::from_list(&attr_args)?;
    
    Ok(parse_individual_event(
        args,
        &event_attrs,
        &event_vis,
        &event_name,
        &event_generics,
        &event_fields
    )?.into())
}