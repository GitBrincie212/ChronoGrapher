use crate::utils::{LIFETIME_UNSUPPORTED_ERR, extract_arg_name, extract_docs, extract_workflow, handle_generics_phantom_data, map_fn_args_pairs, ParsedContextArgument, ParsedArguments};
use crate::workflow::WorkflowSpec;
use crate::workflow::utils::WorkflowTransform;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    FnArg, GenericArgument, PathArguments, ReturnType, Token, Type, TypePath, TypeReference,
    parenthesized, parse_macro_input,
};

const TASKFRAME_CTX_REQUIRED_ERR: &'static str =
    "Function is required to have at least one argument of type \"TaskFrameContext\"";
const FIRST_ARG_NOT_TASKFRAME_CTX_ERR: &'static str =
    "First argument must be of type \"TaskFrameContext\"";
const SIMPLE_IDENTIFIER_FOR_CTX_ERR: &'static str =
    "Expected a simple identifier as argument name for the context";
const FIRST_ARG_REF_TASKFRAME_ERR: &'static str =
    "First argument must be a reference of type \"TaskFrameContext\"";
const METHOD_MACRO_USE_ERR: &'static str =
    "Using the task attribute macro in methods is unsupported";
const INVALID_RETURN_TYPE_ERROR: &'static str =
    "Return type must be of type Result<(), E> in which E is your desired error type";
const FIRST_GENERIC_RETURN_ERR: &'static str =
    "First generic argument of Result must be of type ()";
const SECOND_GENERIC_RETURN_ERR: &'static str =
    "Second generic argument of Result must be an error type";
const ASYNC_REQUIRED_ERR: &'static str = "Function is required to be async";
const ABI_UNSUPPORTED_ERR: &'static str = "ABI functions are unsupported";

pub struct TaskFrameMacroArguments(Option<WorkflowSpec>);

impl Parse for TaskFrameMacroArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut __internal_workflow_spec = None;
        let mut has_entered_loop = false;

        while !input.is_empty() {
            if has_entered_loop {
                let _ = input.parse::<Token![,]>();
            }

            has_entered_loop = true;
            let key: syn::Ident = input.parse()?;
            if key.to_string().as_str() != "__internal_workflow_spec" {
                return Err(syn::Error::new(
                    key.span(),
                    format!("Unknown taskframe argument: {}", key),
                ));
            }

            if __internal_workflow_spec.is_some() {
                return Err(syn::Error::new(
                    key.span(),
                    "Already specified the internal workflow as a parameter (REPORT THIS AS A BUG TO THE DEVELOPERS)",
                ));
            }

            let content;
            parenthesized!(content in input);
            __internal_workflow_spec = Some(content.parse()?);
        }

        let _ = input.parse::<Token![,]>();

        Ok(Self(__internal_workflow_spec))
    }
}

fn extract_arguments(fn_args: &mut Punctuated<FnArg, Comma>) -> syn::Result<(ParsedContextArgument, ParsedArguments)> {
    let mut fn_args = fn_args.pairs_mut();
    let ctx_arg = fn_args.next().ok_or(syn::Error::new(
        proc_macro2::Span::call_site(),
        TASKFRAME_CTX_REQUIRED_ERR,
    ))?;

    let (ctx_name, ctx_type) = match ctx_arg.value() {
        FnArg::Typed(pt) => {
            let arg_name = extract_arg_name(pt, SIMPLE_IDENTIFIER_FOR_CTX_ERR)?;

            match &*pt.ty {
                Type::Reference(TypeReference { elem, .. }) => {
                    let elem = elem.as_ref();
                    let Type::Path(TypePath { path, .. }) = elem else {
                        return Err(syn::Error::new_spanned(
                            &pt.ty,
                            FIRST_ARG_NOT_TASKFRAME_CTX_ERR,
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
                            FIRST_ARG_NOT_TASKFRAME_CTX_ERR,
                        ));
                    }
                }

                _ => {
                    return Err(syn::Error::new_spanned(&pt.ty, FIRST_ARG_REF_TASKFRAME_ERR));
                }
            }

            (arg_name, &*pt.ty)
        }

        FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(ctx_arg, METHOD_MACRO_USE_ERR));
        }
    };

    Ok((
        (ctx_name.clone(), ctx_type.clone()),
        map_fn_args_pairs(&mut fn_args)?
    ))
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
        return Err(syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR));
    };

    let segment = path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR))?;

    if segment.ident != "Result" {
        return Err(syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR));
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR));
    };

    let mut args_iter = args.args.iter();
    let first = args_iter
        .next()
        .ok_or_else(|| syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR))?;

    match first {
        GenericArgument::Type(Type::Tuple(tuple)) if tuple.elems.is_empty() => {}
        _ => {
            return Err(syn::Error::new_spanned(first, FIRST_GENERIC_RETURN_ERR));
        }
    }

    let err_ty = match args_iter.next() {
        Some(GenericArgument::Type(ty)) => ty.clone(),
        _ => {
            return Err(syn::Error::new_spanned(ty, SECOND_GENERIC_RETURN_ERR));
        }
    };

    if args_iter.next().is_some() {
        return Err(syn::Error::new_spanned(ty, INVALID_RETURN_TYPE_ERROR));
    }

    Ok(err_ty)
}

fn derive_with_generics(
    name: &syn::Ident,
    fn_sig: &syn::Signature,
) -> syn::Result<(
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<Punctuated<proc_macro2::TokenStream, Comma>>,
)> {
    let (impl_end_name, phantom_data, normalized_type_params) =
        handle_generics_phantom_data(name, fn_sig)?;

    let generics = &fn_sig.generics;
    let (phantom_data_init, derives) = if phantom_data.is_some() {
        (
            quote! { Self(std::marker::PhantomData) },
            quote! {
                impl #generics Clone for #impl_end_name {
                    fn clone(&self) -> Self {
                        Self(std::marker::PhantomData)
                    }
                }

                impl #generics Copy for #impl_end_name {}
            },
        )
    } else {
        (quote! { Self }, quote! { #[derive(Default, Clone, Copy)] })
    };

    Ok((
        derives,
        impl_end_name,
        phantom_data_init,
        phantom_data,
        normalized_type_params,
    ))
}

pub fn taskframe(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut macro_args = match syn::parse2::<TaskFrameMacroArguments>(attrs.into()) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut input = parse_macro_input!(item as syn::ItemFn);
    let fn_sig = &mut input.sig;
    let fn_name = fn_sig.ident.clone();
    let fn_block = &mut input.block.into_token_stream();
    let fn_vis = &input.vis;
    let fn_args = &mut fn_sig.inputs;
    let fn_output = &fn_sig.output;

    let (
        (ctx_name, ctx_type),
        (arg_names, arg_types)
    ) = match extract_arguments(fn_args) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let result = match extract_error(fn_output) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    if fn_sig.asyncness.is_none() {
        return syn::Error::new(proc_macro2::Span::call_site(), ASYNC_REQUIRED_ERR)
            .into_compile_error()
            .into();
    }

    if fn_sig.abi.is_some() {
        return syn::Error::new(proc_macro2::Span::call_site(), ABI_UNSUPPORTED_ERR)
            .into_compile_error()
            .into();
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
        return syn::Error::new(lt.span(), LIFETIME_UNSUPPORTED_ERR)
            .into_compile_error()
            .into();
    }

    let (derives, impl_end_name, standalone_init, phantom_data, normalized_type_params) =
        match derive_with_generics(&fn_name, &*fn_sig) {
            Ok(res) => res,
            Err(e) => return e.to_compile_error().into(),
        };

    let expanded = normalized_type_params
        .clone()
        .map(|value| quote! { < #value > });

    let temp = expanded.clone().map(|value| quote! { :: #value });

    let docs = extract_docs(&*input.attrs);
    match extract_workflow(&*input.attrs, &mut macro_args.0, |x| {
        WorkflowSpec::parse.parse2(x)
    }) {
        Ok(()) => {}
        Err(e) => return e.to_compile_error().into(),
    }

    let mut expanded_workflow_init = quote! { #fn_name #temp ::single() };
    let mut expanded_workflow_type = quote! { #fn_name #temp };
    if let Some(workflow_spec) = macro_args.0 {
        for primitive in workflow_spec.0.iter() {
            expanded_workflow_init = primitive.transform(expanded_workflow_init);
            expanded_workflow_type = primitive.get_type(expanded_workflow_type);
        }
    }

    quote! {
        #(#docs)*
        #derives
        #fn_vis struct #fn_name #generics #phantom_data #where_clause;

        impl #generics #impl_end_name {
            pub fn single() -> Self {
                #standalone_init
            }

            pub fn workflow() -> <Self as ::chronographer::task::frames::TaskFrame>::Workflow {
                #expanded_workflow_init
            }
        }

        impl #generics ::chronographer::task::frames::TaskFrame for #impl_end_name {
            type Args = (#arg_types);
            type Error = #result;
            type Workflow = #expanded_workflow_type;

            async fn execute(
                &self,
                #ctx_name: #ctx_type,
                args: &<#fn_name #expanded as ::chronographer::task::frames::TaskFrame>::Args
            ) -> Result<(), Self::Error> {
                let (#arg_names) = args;
                #fn_block
            }
        }
    }
    .into()
}
