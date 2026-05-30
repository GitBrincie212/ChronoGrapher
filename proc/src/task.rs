use proc_macro::TokenStream;
use darling::ast::NestedMeta;
use darling::FromMeta;
use quote::quote;
use syn::parse_macro_input;
use crate::utils::{extract_docs, extract_workflow, handle_generics_phantom_data};

#[derive(Debug, FromMeta)]
struct TaskMacroArguments {
    schedule: syn::Expr,

    #[darling(default)]
    singleton: bool,

    taskframe_name_override: Option<syn::Ident>,
    task_name_override: Option<syn::Ident>,
}

pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemFn);

    // TODO: Find a way to remove this boilerplate pattern
    let attr_args: Vec<NestedMeta> = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return darling::Error::from(e).write_errors().into();
        }
    };

    let args = match TaskMacroArguments::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let fn_block = &input.block;
    let fn_sig = &mut input.sig;
    let mut fn_name = fn_sig.ident.clone();
    let fn_args = &fn_sig.inputs;
    let generics = &fn_sig.generics;
    let where_clause = &generics.where_clause;
    let fn_return = &fn_sig.output;
    let fn_abi = &fn_sig.abi;
    let fn_unsafe = &fn_sig.unsafety;
    let fn_vis = &input.vis;

    let schedule = args.schedule;
    let is_singleton = args.singleton;
    let stringified_fn_name = fn_name.to_string();
    if stringified_fn_name.to_lowercase().ends_with("task") {
        fn_name = syn::Ident::new(&stringified_fn_name[..stringified_fn_name.len() - 4], fn_name.span())
    }
    
    let taskframe_name = args.taskframe_name_override
        .unwrap_or(syn::Ident::new(&format!("{fn_name}TaskFrame"), fn_name.span()));

    let task_name = args.task_name_override
        .unwrap_or(syn::Ident::new(&format!("{fn_name}Task"), fn_name.span()));

    let (
        impl_end_name,
        phantom_data,
        normalized_type_params
    ) = match handle_generics_phantom_data(&task_name, &*fn_sig) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded_normalized_type_params = normalized_type_params
        .map(|value| quote! {
            < #value >
        });

    let temp = expanded_normalized_type_params
        .clone()
        .map(|value| quote! {
            #value ::
        });

    let mut workflow_toks = None;
    match extract_workflow(&*input.attrs, &mut workflow_toks, |x| x) {
        Ok(()) => {},
        Err(e) => return e.to_compile_error().into()
    };

    let taskframe_creation_method = if workflow_toks.is_some() {
        quote! { workflow }
    } else { quote! { default }};
    let task_creation = quote! {
        chronographer::task::Task::new(
            #taskframe_name:: #temp #taskframe_creation_method(),
            #schedule
        )
    };

    let docs = extract_docs(&*input.attrs);

    let expanded_workflow_toks = workflow_toks.map(|x| quote! { ,__internal_workflow_spec = (#x)});
    let mut expanded_method_init_logic = task_creation.clone();
    let mut task_method_name = syn::Ident::new("new", proc_macro2::Span::call_site());
    let mut task_method_return_type = quote! { chronographer::task::Task<#taskframe_name #expanded_normalized_type_params> };
    if is_singleton {
        if !fn_sig.generics.params.is_empty() {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "Generics in singleton Tasks are currently unsupported, \
                    manually assemble your own Task or find another way to circumvent this limitation"
            ).to_compile_error().into()
        }

        expanded_method_init_logic = quote! {
            static INSTANCE: std::sync::OnceLock<
            chronographer::task::Task<#taskframe_name #expanded_normalized_type_params>
            > = std::sync::OnceLock::new();

            INSTANCE.get_or_init(|| #task_creation)
        };

        task_method_name = syn::Ident::new("instance", proc_macro2::Span::call_site());
        task_method_return_type = quote! { &'static chronographer::task::Task<#taskframe_name #expanded_normalized_type_params> };
    }

    quote! {
        #(#docs)*
        #fn_vis struct #task_name #generics #phantom_data #where_clause;

        impl #generics #impl_end_name {
            pub fn #task_method_name() -> #task_method_return_type {
                #expanded_method_init_logic
            }
        }

        #[chronographer::taskframe(
            name_override = #taskframe_name
            #expanded_workflow_toks
        )]
        #fn_vis async #fn_abi #fn_unsafe fn #fn_name #generics (#fn_args) #fn_return #where_clause #fn_block
    }.into()
}