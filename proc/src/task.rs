use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Meta, Token};
use syn::punctuated::Punctuated;

#[derive(Debug)]
struct TaskProcAttrArgs {
    schedule: syn::Expr,
    singleton: bool,
}

impl TaskProcAttrArgs {
    fn from_meta_list(
        metas: Punctuated<Meta, Token![,]>,
    ) -> syn::Result<Self> {
        let mut schedule = None;
        let mut singleton = true;

        for meta in metas {
            match meta {
                Meta::NameValue(nv) => {
                    let is_schedule_param = nv.path.is_ident("schedule");
                    let is_singleton_param = nv.path.is_ident("singleton");

                    if !is_schedule_param && !is_singleton_param {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            "Unknown attribute parameter, use either \"singleton\" or \"schedule\"",
                        ));
                    }

                    if is_schedule_param {
                        schedule = Some(nv.value);
                        continue;
                    }

                    if let syn::Expr::Lit(exprlit) = &nv.value
                        && let syn::Lit::Bool(boolean) = &exprlit.lit {
                        singleton = boolean.value;
                        continue;
                    }

                    return Err(syn::Error::new_spanned(
                        nv.value,
                        "Singleton must be a boolean literal",
                    ));
                }

                Meta::Path(path) => {
                    if !path.is_ident("singleton") {
                        return Err(syn::Error::new_spanned(
                            path,
                            "Unknown attribute flag, did you mean to use \"singleton\"?",
                        ));
                    }

                    singleton = true;
                }

                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "Unsupported attribute syntax, use either \"singleton\" or \"schedule\" attribute parameters",
                    ));
                }
            }
        }

        let schedule = schedule.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required ``schedule`` attribute parameter",
            )
        })?;

        Ok(Self {
            schedule,
            singleton,
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
    let fn_name = &mut fn_sig.ident;
    let fn_args = &fn_sig.inputs;
    let fn_return = &fn_sig.output;
    let fn_vis = &input.vis;

    let schedule = args.schedule;
    let is_singleton = args.singleton;
    let stringified_fn_name = fn_name.to_string();
    if stringified_fn_name.to_lowercase().ends_with("task") {
        *fn_name = syn::Ident::new(&stringified_fn_name[..stringified_fn_name.len() - 4], fn_name.span())
    }
    
    let taskframe_name = syn::Ident::new(&format!("{fn_name}TaskFrame"), fn_name.span());
    let task_name = syn::Ident::new(&format!("{fn_name}Task"), fn_name.span());

    let task_creation = quote! {
        chronographer::task::Task::new(
            #taskframe_name::default(),
            #schedule
        )
    };

    let expanded_frame = quote! {
        #[chronographer::taskframe]
        #fn_vis async fn #fn_name(#fn_args) #fn_return #fn_block
    };

    if is_singleton {
        return TokenStream::from(quote! {
            #fn_vis struct #task_name;

            impl #task_name {
                pub fn instance() -> &'static chronographer::task::Task<#taskframe_name> {
                    static INSTANCE: std::sync::OnceLock<chronographer::task::Task<#taskframe_name>> = std::sync::OnceLock::new();

                    INSTANCE.get_or_init(|| #task_creation)
                }
            }

            #expanded_frame
        });
    }

    TokenStream::from(quote! {
        #[derive(Default, Clone, Copy)]
        #fn_vis struct #task_name;

        impl #task_name {
            pub fn new() -> chronographer::task::Task<#taskframe_name> {
                #task_creation
            }
        }

        #expanded_frame
    })
}