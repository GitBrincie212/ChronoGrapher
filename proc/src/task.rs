use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Meta, Token};
use syn::punctuated::Punctuated;
use crate::utils::{extract_docs, handle_generics_phantom_data};

const UNKNOWN_ATTRIBUTE_PARAM_ERR: &'static str =
    "Unknown attribute parameter, use either \"non_singleton\", \
    \"schedule\", \"taskframe_override\" or \"task_override\"";

const TASKFRAME_NAME_OVERRIDE_SPECIFY_ERR: &'static str = "Already specified a TaskFrame name override parameter";
const TASK_NAME_OVERRIDE_SPECIFY_ERR: &'static str = "Already specified a Task name override parameter";
const TASKFRAME_NAME_OVERRIDE_STR_ERR: &'static str = "TaskFrame name override parameter must be a string literal";
const TASK_NAME_OVERRIDE_STR_ERR: &'static str = "Task name override parameter must be a string literal";
const NON_SINGLETON_FLAG_ERR: &'static str = "Non-singleton parameter has to used as a flag only";
const UNKNOWN_ATTRIBUTE_FLAG_ERR: &'static str = "Unknown attribute flag, did you mean to use \"non_singleton\"?";
const MISSING_REQUIRED_SCHEDULE_ERR: &'static str = "Missing required \"schedule\" attribute parameter";

const SINGLETON_GENERIC_LIMITATION_ERR: &'static str =
    "Generics in singleton Tasks are currently unsupported, \
    manually assemble your own Task or find another way to circumvent this limitation";

#[derive(Debug)]
struct TaskProcAttrArgs {
    schedule: syn::Expr,
    singleton: bool,
    taskframe_name_override: Option<syn::Ident>,
    task_name_override: Option<syn::Ident>,
}

macro_rules! read_identifier {
    ($nv: ident, $param: expr, $target: expr, $err_override: ident, $err_str: ident) => {{
        if $param {
            if let syn::Expr::Path(exprlit) = &$nv.value
            && let Some(ident) = exprlit.path.get_ident()
            {
                $target = Some(ident.clone());
                continue;
            } else if $target.is_some() {
                return Err(syn::Error::new_spanned(
                    $nv.value,
                    $err_override
                ));
            }
            
            return Err(syn::Error::new_spanned(
                $nv.value,
                $err_str,
            ));
        }
    }};
}

impl TaskProcAttrArgs {
    fn from_meta_list(
        metas: Punctuated<Meta, Token![,]>,
    ) -> syn::Result<Self> {
        let mut schedule = None;
        let mut singleton = true;
        let mut taskframe_name_override = None;
        let mut task_name_override = None;

        for meta in metas {
            match meta {
                Meta::NameValue(nv) => {
                    let is_schedule_param = nv.path.is_ident("schedule");
                    let is_taskframe_name_override = nv.path.is_ident("taskframe_override");
                    let is_task_name_override = nv.path.is_ident("task_override");

                    if !is_schedule_param && !is_taskframe_name_override && !is_task_name_override {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            UNKNOWN_ATTRIBUTE_PARAM_ERR
                        ));
                    }

                    read_identifier!(
                        nv, is_taskframe_name_override, taskframe_name_override, 
                        TASKFRAME_NAME_OVERRIDE_SPECIFY_ERR, TASKFRAME_NAME_OVERRIDE_STR_ERR
                    );

                    read_identifier!(
                        nv, is_task_name_override, task_name_override, 
                        TASK_NAME_OVERRIDE_SPECIFY_ERR, TASK_NAME_OVERRIDE_STR_ERR
                    );

                    if nv.path.is_ident("non_singleton") {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            NON_SINGLETON_FLAG_ERR
                        ));
                    }

                    schedule = Some(nv.value);
                }

                Meta::Path(path) => {
                    if !path.is_ident("non_singleton") {
                        return Err(syn::Error::new_spanned(
                            path,
                            UNKNOWN_ATTRIBUTE_FLAG_ERR
                        ));
                    }

                    singleton = false;
                }

                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        UNKNOWN_ATTRIBUTE_PARAM_ERR
                    ));
                }
            }
        }

        let schedule = schedule.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                MISSING_REQUIRED_SCHEDULE_ERR
            )
        })?;

        Ok(Self {
            schedule,
            singleton,
            taskframe_name_override,
            task_name_override,
        })
    }
}

pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(
        attr with Punctuated::<Meta, Token![,]>::parse_terminated
    );

    let args = match TaskProcAttrArgs::from_meta_list(attr_args) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut input = parse_macro_input!(item as syn::ItemFn);
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

    let task_creation = quote! {
        chronographer::task::Task::new(
            #taskframe_name:: #temp default(),
            #schedule
        )
    };

    let docs = extract_docs(&*input.attrs);

    let expanded_frame = quote! {
        #[chronographer::taskframe(name_override = #taskframe_name)]
        #fn_vis async #fn_abi #fn_unsafe fn #fn_name #generics (#fn_args) #fn_return #where_clause #fn_block
    };

    if is_singleton {
        if !fn_sig.generics.params.is_empty() {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                SINGLETON_GENERIC_LIMITATION_ERR
            ).to_compile_error().into()
        }

        return TokenStream::from(quote! {
            #(#docs)*
            #fn_vis struct #task_name #generics #phantom_data #where_clause;

            impl #generics #impl_end_name {
                pub fn instance() -> &'static chronographer::task::Task<#taskframe_name #expanded_normalized_type_params> {
                    static INSTANCE: std::sync::OnceLock<
                        chronographer::task::Task<#taskframe_name #expanded_normalized_type_params>
                    > = std::sync::OnceLock::new();

                    INSTANCE.get_or_init(|| #task_creation)
                }
            }

            #expanded_frame
        });
    }

    TokenStream::from(quote! {
        #(#docs)*
        #fn_vis struct #task_name #generics #phantom_data #where_clause;

        impl #generics #impl_end_name {
            pub fn new() -> chronographer::task::Task<#taskframe_name #expanded_normalized_type_params> {
                #task_creation
            }
        }

        #expanded_frame
    })
}