use crate::utils::{extract_docs, extract_annotation, handle_generics_phantom_data};
use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::parse_macro_input;
use crate::hook::hook_attachment_annotation::HookAnnotationArguments;

#[derive(Debug, FromMeta)]
struct TaskMacroArguments {
    schedule: syn::Expr,

    singleton: Option<bool>,

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
    let is_singleton = args.singleton.unwrap_or(true);
    let stringified_fn_name = fn_name.to_string();
    if stringified_fn_name.to_lowercase().ends_with("task") {
        fn_name = syn::Ident::new(
            &stringified_fn_name[..stringified_fn_name.len() - 4],
            fn_name.span(),
        )
    }

    let taskframe_name = args.taskframe_name_override.unwrap_or(syn::Ident::new(
        &format!("{fn_name}TaskFrame"),
        fn_name.span(),
    ));

    let task_name = args
        .task_name_override
        .unwrap_or(syn::Ident::new(&format!("{fn_name}Task"), fn_name.span()));

    let (impl_end_name, phantom_data, normalized_type_params) =
        match handle_generics_phantom_data(&task_name, &*fn_sig) {
            Ok(res) => res,
            Err(e) => return e.to_compile_error().into(),
        };

    let expanded_normalized_type_params = normalized_type_params.map(|value| {
        quote! {
            < #value >
        }
    });

    let temp = expanded_normalized_type_params.clone().map(|value| {
        quote! {
            #value ::
        }
    });

    let mut workflow_toks = None;
    match extract_annotation(&*input.attrs, "Workflow", &mut workflow_toks, |x| Ok(x)) {
        Ok(()) => {}
        Err(e) => return e.to_compile_error().into(),
    };

    let mut hook_annotation_parsed = None;
    match extract_annotation(&*input.attrs, "Hook", &mut hook_annotation_parsed, |x| {
        HookAnnotationArguments::parse.parse2(x)
    }) {
        Ok(()) => {}
        Err(e) => return e.to_compile_error().into(),
    };

    let taskframe_creation_method = if workflow_toks.is_some() {
        quote! { workflow }
    } else { quote! { single } };

    let task_creation = quote! {
        let task = chronographer::task::Task::new(
            #taskframe_name:: #temp #taskframe_creation_method(),
            #schedule
        );

        #hook_annotation_parsed

        task
    };

    let docs = extract_docs(&*input.attrs);

    let expanded_workflow_toks = workflow_toks.map(|x| quote! { __internal_workflow_spec(#x)});
    let mut expanded_method_init_logic = task_creation.clone();
    let mut task_method_name = syn::Ident::new("new", proc_macro2::Span::call_site());
    let workflow = quote! { <#taskframe_name #temp as ::chronographer::task::frames::TaskFrame>::Workflow };
    let mut task_method_return_type = quote! { ::chronographer::task::Task<#workflow> };

    let constructor_async = hook_annotation_parsed
        .as_ref()
        .map(|_| quote! { async });
    if is_singleton {
        if !fn_sig.generics.params.is_empty() {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "Generics in singleton Tasks are currently unsupported, \
                    manually assemble your own Task or find another way to circumvent this limitation"
            ).to_compile_error().into();
        }

        let (
            singleton_primitive_type,
            await_expansion,
            method_type
        ) = if hook_annotation_parsed.is_none() {
            (quote! { std::sync::OnceLock }, None, quote! { new })
        } else { (quote! { tokio::sync::OnceCell }, Some(quote! { .await }), quote! { const_new })};

        expanded_method_init_logic = quote! {
            static INSTANCE: #singleton_primitive_type<
            chronographer::task::Task<#workflow>
            > = #singleton_primitive_type::#method_type();

            INSTANCE.get_or_init(|| #constructor_async { #task_creation }) #await_expansion
        };

        task_method_name = syn::Ident::new("instance", proc_macro2::Span::call_site());
        task_method_return_type = quote! { &'static ::chronographer::task::Task<#workflow> };
    }

    quote! {
        #(#docs)*
        #fn_vis struct #task_name #generics #phantom_data #where_clause;

        impl #generics #impl_end_name {
            pub #constructor_async fn #task_method_name() -> #task_method_return_type {
                #expanded_method_init_logic
            }
        }

        #[chronographer::taskframe(#expanded_workflow_toks)]
        #fn_vis async #fn_abi #fn_unsafe fn #taskframe_name #generics (#fn_args) #fn_return #where_clause #fn_block
    }.into()
}
