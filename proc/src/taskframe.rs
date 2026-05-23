use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, GenericArgument, Pat, PatType, PathArguments, ReturnType, Token, Type, TypePath, TypeReference};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use crate::utils::{extract_docs, handle_generics_phantom_data, LIFETIME_UNSUPPORTED_ERR};

const NAME_OVERRIDE_IDENTIFIER_ERR: &'static str = "Name override parameter must be a simple identifier";
const UNKNOWN_ATTRIBUTE_PARAM_ERR: &'static str = "Unknown attribute parameter, did you mean to use \"name_override\"?";
const TASKFRAME_CTX_REQUIRED_ERR: &'static str = "Function is required to have at least one argument of type \"TaskFrameContext\"";
const FIRST_ARG_NOT_TASKFRAME_CTX_ERR: &'static str = "First argument must be of type \"TaskFrameContext\"";
const SIMPLE_IDENTIFIER_FOR_CTX_ERR: &'static str = "Expected a simple identifier as argument name for the context";
const SIMPLE_IDENTIFIER_FOR_ARG_ERR: &'static str = "Expected a simple identifier as an argument name";
const FIRST_ARG_REF_TASKFRAME_ERR: &'static str = "First argument must be a reference of type \"TaskFrameContext\"";
const METHOD_MACRO_USE_ERR: &'static str = "Using the task attribute macro in methods is unsupported";
const USE_OF_REF_SELF_ERR: &'static str = "Invalid syntax, cannot use &self or &mut self";
const INVALID_RETURN_TYPE_ERROR: &'static str = "Return type must be of type Result<(), E> in which E is your desired error type";
const FIRST_GENERIC_RETURN_ERR: &'static str = "First generic argument of Result must be of type ()";
const SECOND_GENERIC_RETURN_ERR: &'static str = "Second generic argument of Result must be an error type";
const ASYNC_REQUIRED_ERR: &'static str = "Function is required to be async";
const ABI_UNSUPPORTED_ERR: &'static str = "ABI functions are unsupported";

#[derive(Debug)]
pub struct TaskFrameProcAttrArgs(Option<syn::Ident>);

impl TaskFrameProcAttrArgs {
    fn from_meta_list(
        metas: Punctuated<syn::Meta, Token![,]>,
    ) -> syn::Result<Self> {
        let mut override_val = None;

        for meta in metas {
            match meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("name_override") => {
                    let syn::Expr::Path(exprpath) = &nv.value else {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            NAME_OVERRIDE_IDENTIFIER_ERR
                        ));
                    };

                    let Some(ident) = exprpath.path.get_ident() else {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            NAME_OVERRIDE_IDENTIFIER_ERR
                        ));
                    };

                    override_val = Some(ident.clone());
                    continue;
                }

                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        UNKNOWN_ATTRIBUTE_PARAM_ERR
                    ));
                }
            }
        }

        Ok(Self(override_val))
    }
}

fn extract_arg_name<'a>(pt: &'a PatType, err: &str) -> syn::Result<&'a proc_macro2::Ident> {
    match &*pt.pat {
        Pat::Ident(pat_ident) => Ok(&pat_ident.ident),
        _ => {
            Err(syn::Error::new_spanned(
                &pt.pat,
                err,
            ))
        }
    }
}

fn extract_arguments(
    fn_args: &mut Punctuated<FnArg, Comma>
) -> syn::Result<(Punctuated<proc_macro2::Ident, Comma>, Punctuated<Type, Comma>)> {
    let mut fn_args = fn_args.pairs_mut();
    let ctx_arg = fn_args.next()
        .ok_or(
            syn::Error::new(
                proc_macro2::Span::call_site(),
                TASKFRAME_CTX_REQUIRED_ERR
            )
        )?;

    let (ctx_name, ctx_type) = match ctx_arg.value() {
        FnArg::Typed(pt) => {
            let arg_name = extract_arg_name(pt, SIMPLE_IDENTIFIER_FOR_CTX_ERR)?;

            match &*pt.ty {
                Type::Reference(TypeReference { elem, .. }) => {
                    let elem = elem.as_ref();
                    let Type::Path(TypePath { path, .. }) = elem else {
                        return Err(syn::Error::new_spanned(
                            &pt.ty,
                            FIRST_ARG_NOT_TASKFRAME_CTX_ERR
                        ));
                    };

                    let is_ctx = path
                        .segments
                        .last()
                        .map(|seg| seg.ident == "TaskFrameContext")
                        .unwrap_or(false);

                    if !is_ctx {
                        return Err(syn::Error::new_spanned(
                            &pt.ty,
                            FIRST_ARG_NOT_TASKFRAME_CTX_ERR
                        ));
                    }
                }

                _ => {
                    return Err(syn::Error::new_spanned(
                        &pt.ty,
                        FIRST_ARG_REF_TASKFRAME_ERR
                    ));
                }
            }

            (arg_name, &*pt.ty)
        }

        FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(
                ctx_arg,
                METHOD_MACRO_USE_ERR
            ));
        }
    };


    let mut names = Punctuated::new();
    let mut types = Punctuated::new();
    while let Some(argument) = fn_args.next() {
        match argument.value() {
            FnArg::Typed(pt) => {
                let arg_name = extract_arg_name(pt, SIMPLE_IDENTIFIER_FOR_ARG_ERR)?;
                let arg_type = &*pt.ty;
                names.push(arg_name.clone());
                types.push(arg_type.clone());
            }

            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    ctx_arg,
                    USE_OF_REF_SELF_ERR
                ));
            }
        }
    }

    names.push(ctx_name.clone());
    types.push(ctx_type.clone());
    Ok((names, types))
}

fn extract_error(return_type: &ReturnType) -> syn::Result<Type> {
    let ty = match return_type {
        ReturnType::Type(_, ty) => ty,
        ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                return_type,
                INVALID_RETURN_TYPE_ERROR,
            ));
        }
    };

    let Type::Path(TypePath { path, .. }) = ty.as_ref() else {
        return Err(syn::Error::new_spanned(
            ty,
            INVALID_RETURN_TYPE_ERROR,
        ));
    };

    let segment = path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR)
    })?;


    if segment.ident != "Result" {
        return Err(syn::Error::new_spanned(
            ty,
            INVALID_RETURN_TYPE_ERROR
        ));
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            INVALID_RETURN_TYPE_ERROR
        ));
    };

    let mut args_iter = args.args.iter();
    let first = args_iter.next().ok_or_else(|| {
        syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR)
    })?;

    match first {
        GenericArgument::Type(Type::Tuple(tuple)) if tuple.elems.is_empty() => {}
        _ => {
            return Err(syn::Error::new_spanned(
                first,
                FIRST_GENERIC_RETURN_ERR
            ));
        }
    }

    let err_ty = match args_iter.next() {
        Some(GenericArgument::Type(ty)) => ty.clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                ty,
                SECOND_GENERIC_RETURN_ERR
            ));
        }
    };

    if args_iter.next().is_some() {
        return Err(syn::Error::new_spanned(
            ty,
            INVALID_RETURN_TYPE_ERROR,
        ));
    }

    Ok(err_ty)
}

fn derive_with_generics(name: &syn::Ident, fn_sig: &syn::Signature) -> syn::Result<(
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<Punctuated<proc_macro2::TokenStream, Comma>>
)> {
    let (
        impl_end_name,
        phantom_data,
        normalized_type_params
    ) = handle_generics_phantom_data(name, fn_sig)?;

    let generics = &fn_sig.generics;
    let derives = if phantom_data.is_some() {
        quote! {
            impl #generics Default for #impl_end_name {
                fn default() -> Self {
                    Self(std::marker::PhantomData)
                }
            }

            impl #generics Clone for #impl_end_name {
                fn clone(&self) -> Self {
                    Self(std::marker::PhantomData)
                }
            }

            impl #generics Copy for #impl_end_name {}
        }
    } else { quote! { #[derive(Default, Clone, Copy)] } };

    Ok((derives, impl_end_name, phantom_data, normalized_type_params))
}

pub fn taskframe(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemFn);

    let attr_args = parse_macro_input!(
        attrs with Punctuated::<syn::Meta, Token![,]>::parse_terminated
    );

    let name_override = match TaskFrameProcAttrArgs::from_meta_list(attr_args) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    }.0;

    let fn_sig = &mut input.sig;
    let fn_name = &mut fn_sig.ident;
    let fn_block = &mut input.block.into_token_stream();
    let fn_vis = &input.vis;
    let fn_args = &mut fn_sig.inputs;
    let fn_output = &fn_sig.output;

    let (mut arg_names, mut arg_types) = match extract_arguments(fn_args) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let ctx_name = arg_names.pop().unwrap();
    let ctx_type = arg_types.pop().unwrap();

    let result = match extract_error(fn_output) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let stringified_fn_name = fn_name.to_string();
    if stringified_fn_name.to_lowercase().ends_with("frame") {
        *fn_name = syn::Ident::new(&stringified_fn_name[..stringified_fn_name.len() - 5], fn_name.span())
    }
    
    let taskframe_name = name_override
        .unwrap_or(syn::Ident::new(&format!("{fn_name}TaskFrame"), fn_name.span()));


    if fn_sig.asyncness.is_none() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            ASYNC_REQUIRED_ERR,
        ).into_compile_error().into();
    }

    if fn_sig.abi.is_some() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            ABI_UNSUPPORTED_ERR
        ).into_compile_error().into();
    }

    if fn_sig.unsafety.is_some() {
        *fn_block = quote! {
            {
                unsafe { #fn_block }
            }
        }
    }

    let generics = &fn_sig.generics;
    let where_clause = &fn_sig.generics.where_clause;

    if let Some(lt) = generics.lifetimes().next() {
        return syn::Error::new(
            lt.span(),
            LIFETIME_UNSUPPORTED_ERR,
        ).into_compile_error().into();
    }

    let (
        derives,
        impl_end_name,
        phantom_data,
        normalized_type_params
    ) = match derive_with_generics(&taskframe_name, &*fn_sig) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = normalized_type_params.map(|value| {
        quote! { < #value > }
    });

    let docs = extract_docs(&*input.attrs);

    quote! {
        #(#docs)*
        #derives
        #fn_vis struct #taskframe_name #generics #phantom_data #where_clause;

        impl #generics chronographer::task::TaskFrame for #impl_end_name {
            type Args = (#arg_types);
            type Error = #result;

            async fn execute(
                &self,
                #ctx_name: #ctx_type,
                args: &<#taskframe_name #expanded as chronographer::task::TaskFrame>::Args
            ) -> Result<(), Self::Error> {
                let (#arg_names) = args;
                #fn_block
            }
        }
    }.into()
}