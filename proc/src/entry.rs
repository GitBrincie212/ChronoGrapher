use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, Meta, ReturnType, Token, Type};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use crate::utils::extract_arg_name;

const UNKNOWN_ATTRIBUTE_PARAM_ERR: &'static str = "Unknown attribute parameter, use either \"thread_count\" or \"start_before\"";
const FUNCTION_NOT_MAIN_ERR: &'static str = "Expected function to be a main function and nto something else";
const UNEXPECTED_SELF_ARG_ERR: &'static str = "Unexpected &self / &mut self used in main function";
const SIMPLE_IDENTIFIER_FOR_SCHEDULER_ERR: &'static str = "Expected a simple identifier as argument name for the scheduler";
const EXPECTED_AT_LEAST_ONE_SCHEDULER_ERR: &'static str = "Expected at least one argument for the scheduler to initialize";
const UNEXPECTED_RETURN_TYPE_ERR: &'static str = "Expected either () or a Return<T, E> as return type but got something else";
const START_BEFORE_FLAG_ERR: &'static str = "start_before parameter has to used as a flag only";
const UNKNOWN_ATTRIBUTE_FLAG_ERR: &'static str = "Unknown attribute flag, did you mean to use \"start_before\"?";
const THREAD_COUNT_SPECIFY_ERR: &'static str = "Already specified the thread count parameter";
const THREAD_COUNT_INT_ERR: &'static str = "Thread count parameter must be an integer literal";


#[derive(Debug)]
pub struct EntryProcAttrs {
    thread_count: Option<syn::LitInt>,
    start_before: bool
}

impl EntryProcAttrs {
    fn from_meta_list(
        metas: Punctuated<Meta, Token![,]>,
    ) -> syn::Result<Self> {
        let mut thread_count = None;
        let mut start_before = false;

        for meta in metas {
            match meta {
                Meta::NameValue(nv) => {
                    let is_thread_count_param = nv.path.is_ident("thread_count");
                    let is_start_before_param = nv.path.is_ident("start_before");

                    if !is_thread_count_param && !is_start_before_param {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            UNKNOWN_ATTRIBUTE_PARAM_ERR
                        ));
                    }

                    if is_thread_count_param {
                        if let syn::Expr::Lit(exprlit) = &nv.value
                            && let syn::Lit::Int(ident) = &exprlit.lit
                        {
                            thread_count = Some(ident.clone());
                            continue;
                        } else if thread_count.is_some() {
                            return Err(syn::Error::new_spanned(
                                nv.value,
                                THREAD_COUNT_SPECIFY_ERR,
                            ));
                        }

                        return Err(syn::Error::new_spanned(
                            nv.value,
                            THREAD_COUNT_INT_ERR,
                        ));
                    }

                    return Err(syn::Error::new_spanned(
                        nv.value,
                        START_BEFORE_FLAG_ERR
                    ));
                }

                Meta::Path(path) => {
                    if !path.is_ident("start_before") {
                        return Err(syn::Error::new_spanned(
                            path,
                            UNKNOWN_ATTRIBUTE_FLAG_ERR
                        ));
                    }

                    start_before = true;
                }

                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        UNKNOWN_ATTRIBUTE_PARAM_ERR
                    ));
                }
            }
        }

        Ok(Self {
            thread_count,
            start_before
        })
    }
}

fn argument_schedulers_to_code(
    fn_args: &Punctuated<FnArg, Comma>
) -> syn::Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)> {
    if fn_args.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            EXPECTED_AT_LEAST_ONE_SCHEDULER_ERR
        ));
    }

    let mut scheduler_inits = Vec::with_capacity(fn_args.len());
    let mut scheduler_starts = Vec::with_capacity(fn_args.len());
    for arg in fn_args {
        match arg {
            FnArg::Typed(pat) => {
                let name = extract_arg_name(&pat, SIMPLE_IDENTIFIER_FOR_SCHEDULER_ERR)?;
                let ty = pat.ty.as_ref();

                let expanded_init = quote! { let #name = <#ty as Default>::default(); };
                let expanded_start = quote! { chronographer::prelude::Scheduler::start(&#name).await; };
                scheduler_inits.push(expanded_init);
                scheduler_starts.push(expanded_start);
            }

            _ => {
                return Err(syn::Error::new(
                    arg.span(),
                    UNEXPECTED_SELF_ARG_ERR
                ));
            }
        }
    }

    Ok((scheduler_inits, scheduler_starts))
}

pub fn entry(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let attr_args = parse_macro_input!(
        attrs with Punctuated::<Meta, Token![,]>::parse_terminated
    );

    let args = match EntryProcAttrs::from_meta_list(attr_args) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let fn_sig = &input.sig;
    let fn_name = &fn_sig.ident;
    let fn_return = &fn_sig.output;
    let fn_block = &input.block;

    if fn_name.to_string() != "main" {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            FUNCTION_NOT_MAIN_ERR
        ).to_compile_error().into();
    }

    let (scheduler_inits, scheduler_starts) = match argument_schedulers_to_code(&fn_sig.inputs) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into()
    };

    let thread_method = if let Some(val) = args.thread_count {
        quote! { .worker_threads(#val) }
    } else {
        quote ! { }
    };

    let mut scheduler_start_top_block = quote! { };
    let mut scheduler_start_bottom_block = quote! { #(#scheduler_starts)* };
    if args.start_before {
        scheduler_start_top_block = scheduler_start_bottom_block;
        scheduler_start_bottom_block = quote! {};
    }

    let runtime_init = quote! {
        let rt = tokio::runtime::Builder::new_multi_thread()
                #thread_method
                .enable_all()
                .build()
                .unwrap();
    };

    let expanded = match fn_return {
        ReturnType::Default => quote! {
            fn main() #fn_return {
                #runtime_init

                rt.block_on(async {
                    #(#scheduler_inits)*
                    #scheduler_start_top_block

                    (async #fn_block).await;

                    #scheduler_start_bottom_block
                    tokio::signal::ctrl_c().await.unwrap();
                });
            }
        },

        ReturnType::Type(_, ty) => {
            match ty.as_ref() {
                Type::Path(pt) => {
                    let is_result = match ty.as_ref() {
                        Type::Path(pt) => {
                            pt.path.segments.last().is_some_and(|s| s.ident == "Result")
                        }
                        _ => false,
                    };

                    if !is_result {
                        return syn::Error::new_spanned(
                            pt,
                            UNEXPECTED_RETURN_TYPE_ERR
                        ).to_compile_error().into();
                    }

                    quote! {
                        fn main() #fn_return {
                            #runtime_init

                            let final_res = rt.block_on(async {
                                #(#scheduler_inits)*
                                #scheduler_start_top_block

                                let res: #ty = (async #fn_block).await;
                                let extracted_res = res?;

                                #scheduler_start_bottom_block
                                tokio::signal::ctrl_c().await.unwrap();

                                Ok(extracted_res)
                            });

                            final_res
                        }
                    }
                }

                _ => {
                    return syn::Error::new_spanned(
                        ty,
                        UNEXPECTED_RETURN_TYPE_ERR
                    ).to_compile_error().into();
                }
            }
        }
    };

    expanded.into()
}