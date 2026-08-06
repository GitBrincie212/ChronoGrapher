pub mod hook_attachment_annotation;
pub mod hook_event_annotation;

use proc_macro::TokenStream;
use std::collections::HashSet;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, Token};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::hook::hook_event_annotation::{HookAnnotationMacroArguments, HookItemDefaultField};
use crate::utils::{extract_arg_name, map_fn_args_pairs, ParsedArguments, ParsedContextArgument};

/*
    Out of all macros, this is by far the worst in terms of readability (in my opinion). I'm sorry in advance,
    il try to make it better in the future, but for now as a small prototype it does the job.

    TODO: Improve the code quality in the future
 */

const TASKHOOK_CTX_REQUIRED_ERR: &'static str =
    "Method is required to have at least one argument of type \"TaskHookContext\" after &self";
const TASKHOOK_SELF_REQUIRED_ERR: &'static str =
    "Method is required to have at least two arguments of &self and the other of type \"TaskHookContext\" after";
const EXPECTED_SELF_ARGUMENT_ERR: &'static str =
    "Expected argument of &self but got something else";
const SECOND_ARG_NOT_TASKHOOK_CTX_ERR: &'static str =
    "Second argument must be of type \"TaskFrameContext\"";
const SIMPLE_IDENTIFIER_FOR_CTX_ERR: &'static str =
    "Expected a simple identifier as argument name for the context";
const SECOND_ARG_REF_TASKHOOK_ERR: &'static str =
    "Second argument must be a reference of type \"TaskFrameContext\"";
const ASYNC_REQUIRED_ERR: &'static str = "Method is required to be async";
const ABI_UNSUPPORTED_ERR: &'static str = "ABI functions are unsupported";
const DEFAULT_NOT_ALLOWED_ERR: &'static str =
    "Auto-attach methods are disallowed as the configuration for auto-attachment is disabled";

enum HookMacroArguments {
    Disabled,
    Enabled(syn::Ident),
}

impl Parse for HookMacroArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(HookMacroArguments::Enabled(syn::Ident::new(
                "auto_attach",
                proc_macro2::Span::call_site()
            )));
        }

        let mut negate = false;
        if input.peek(Token![!]) {
            _ = input.parse::<Token![!]>();
            negate = !negate;
        }

        if input.parse::<syn::Ident>()?.to_string().as_str() != "auto_attach" {
            return Err(input.error("Unrecognized hook macro argument identifier. Did you mean to use 'auto_attach'?"));
        }

        if negate {
            if !input.is_empty() {
                return Err(input.error("Unexpected tokens after '!auto_attach'. Did you mean 'auto_attach = ...'?"));
            }

            return Ok(HookMacroArguments::Disabled);
        }

        if input.peek(Token![=]) {
            let _ = input.parse::<Token![=]>()?;
            let name = input.parse::<syn::Ident>()?;

            if !input.is_empty() {
                return Err(input.error("Unexpected tokens after assignment. Expected end of arguments."));
            }

            return Ok(HookMacroArguments::Enabled(name));
        }

        if !input.is_empty() {
            return Err(input.error("Unexpected tokens after 'auto_attach'. Did you mean '!auto_attach' or 'auto_attach = ...'?"));
        }

        Ok(HookMacroArguments::Enabled(syn::Ident::new(
            "auto_attach",
            proc_macro2::Span::call_site()
        )))
    }
}

fn extract_arguments(fn_args: &mut Punctuated<FnArg, Comma>) -> syn::Result<(ParsedContextArgument, ParsedArguments)> {
    let mut fn_args = fn_args.pairs_mut();
    let slf = fn_args.next().ok_or(syn::Error::new(
        proc_macro2::Span::call_site(),
        TASKHOOK_SELF_REQUIRED_ERR,
    ))?;

    let ctx_arg = fn_args.next().ok_or(syn::Error::new(
        proc_macro2::Span::call_site(),
        TASKHOOK_CTX_REQUIRED_ERR,
    ))?;

    match slf.value() {
        FnArg::Receiver(receiver) => {
            if !receiver.reference.is_some()
                || !receiver.mutability.is_none()
                || !receiver.colon_token.is_none() {
                return Err(syn::Error::new_spanned(receiver, EXPECTED_SELF_ARGUMENT_ERR))
            }
        }

        FnArg::Typed(_) => {
            return Err(syn::Error::new_spanned(slf, EXPECTED_SELF_ARGUMENT_ERR))
        }
    }

    let (ctx_name, ctx_type) = match ctx_arg.value() {
        FnArg::Typed(pt) => {
            let arg_name = extract_arg_name(pt, SIMPLE_IDENTIFIER_FOR_CTX_ERR)?;

            match &*pt.ty {
                syn::Type::Reference(syn::TypeReference { elem, .. }) => {
                    let elem = elem.as_ref();
                    let syn::Type::Path(syn::TypePath { path, .. }) = elem else {
                        return Err(syn::Error::new_spanned(
                            &pt.ty,
                            SECOND_ARG_NOT_TASKHOOK_CTX_ERR,
                        ));
                    };

                    let is_ctx = path
                        .segments
                        .last()
                        .map(|seg| seg.ident == "TaskHookContext")
                        .unwrap_or(false);

                    if !is_ctx {
                        return Err(syn::Error::new_spanned(
                            &pt.ty,
                            SECOND_ARG_NOT_TASKHOOK_CTX_ERR,
                        ));
                    }
                }

                _ => {
                    return Err(syn::Error::new_spanned(&pt.ty, SECOND_ARG_REF_TASKHOOK_ERR));
                }
            }

            (arg_name, &*pt.ty)
        }

        FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(ctx_arg, SECOND_ARG_NOT_TASKHOOK_CTX_ERR));
        }
    };

    Ok((
        (ctx_name.clone(), ctx_type.clone()),
        map_fn_args_pairs(&mut fn_args)?
    ))
}

fn push_defaults(
    item_defaults_enabled: bool,
    fn_name: &syn::Ident,
    item_defaults: Punctuated<HookItemDefaultField, Comma>,
    defaults: &mut Vec<TokenStream2>,
    event_expanded: &TokenStream2,
    generics: &Punctuated<syn::GenericParam, Comma>
) -> syn::Result<()> {
    if item_defaults.is_empty() {
        if !item_defaults_enabled {
            return Ok(());
        }

        for generic in generics {
            let syn::GenericParam::Type(ty) = generic else {
                continue;
            };

            if ty.colon_token.is_some() {
                return Err(syn::Error::new_spanned(ty, "Broad generics are not allowed without specifying explicit defaults"));
            }
        }

        defaults.push(quote! { #event_expanded });
        return Ok(());
    }

    let mut encountered_broad_generic = false;
    for def in item_defaults.iter() {
        if def.0.len() != generics.len() {
            return Err(syn::Error::new_spanned(def, "Generic defaults do not match up with the generic parameters"));
        }

        for (generic, supplied) in generics.iter().zip(def.0.iter()) {
            let syn::GenericParam::Type(param) = generic else {
                continue;
            };

            if param.colon_token.is_none() {
                let expected = &param.ident;

                let matches = match supplied {
                    syn::Type::Path(path) => path.path.is_ident(expected),
                    syn::Type::Infer(_) => true,
                    _ => false,
                };

                if !matches {
                    return Err(syn::Error::new_spanned(
                        supplied,
                        format!("Narrow generic must match with {expected} or use _")
                    ));
                }

                continue;
            }

            encountered_broad_generic = true;
        }

        if !encountered_broad_generic {
            return Err(syn::Error::new_spanned(
                item_defaults.first().unwrap(),
                "Cannot specify explicit defaults for only narrow-based generics"
            ));
        }
    }

    if item_defaults_enabled && fn_name != "__anonymous__" {
        defaults.extend(
            item_defaults.iter().map(|def| {
                let args = &def.0;
                quote! {
                    #fn_name < #args >
                }
            })
        );
    } else {
        defaults.extend(
            item_defaults.iter().map(|def| {
                def.0.to_token_stream()
            })
        );
    }

    Ok(())
}

pub fn hook(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = match syn::parse2::<HookMacroArguments>(attrs.into()) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut input = parse_macro_input!(item as syn::ItemImpl);
    let taskhook_name = match input.self_ty.as_ref() {
        syn::Type::Path(tp) => &tp.path,
        ty => return syn::Error::new_spanned(ty, "Expected a simple identifier in impl block's name but got something else")
            .into_compile_error()
            .into()
    };

    if input.trait_.is_some() {
        return syn::Error::new_spanned(input, "#[hook] Macro not allowed in trait implementations")
            .into_compile_error()
            .into()
    }

    let mut taskhook_impl_end_expanded = quote! { #taskhook_name };
    let parent_generics = &input.generics;
    if !parent_generics.params.is_empty() {
        let param_names = parent_generics.params.iter()
            .map(|param| match param {
                syn::GenericParam::Type(ty) => {
                    let ty_name = &ty.ident;
                    quote! { #ty_name }
                },

                syn::GenericParam::Lifetime(lt) => {
                    let lifetime = &lt.lifetime;
                    quote! { #lifetime }
                }

                syn::GenericParam::Const(ct) => {
                    let ct_name = &ct.ident;
                    quote! { #ct_name }
                }
            })
            .collect::<Punctuated<_, Comma>>();
        taskhook_impl_end_expanded = quote! { #taskhook_impl_end_expanded <#param_names> }
    }

    let mut encountered_default = false;
    let mut recorded_events = HashSet::new();
    let mut defaults = Vec::new();
    let mut expanded_listeners = quote! { };
    for item in &mut input.items {
        let syn::ImplItem::Fn(func) = item else {
            return syn::Error::new_spanned(input, "Non-method / Non-function item found within hook implementation")
                .into_compile_error()
                .into()
        };

        let fn_sig = &mut func.sig;
        let fn_name = &fn_sig.ident;
        let fn_block = &mut (&func.block).into_token_stream();
        let fn_return = &fn_sig.output;
        let fn_generics = &fn_sig.generics;
        let mut fn_name_expanded = fn_name.to_token_stream();
        let mut item_defaults = Punctuated::new();
        let mut item_defaults_enabled = matches!(macro_args, HookMacroArguments::Enabled(_));

        match HookAnnotationMacroArguments::parse_attrs(&func.attrs) {
            Ok(Some(args)) => {
                let args_defaults = args.defaults;
                item_defaults_enabled = args_defaults.0;
                item_defaults = args_defaults.1;

                for def in &item_defaults {
                    if def.0.len() != fn_generics.params.len() {
                        return syn::Error::new_spanned(
                            &def.0,
                            format!(
                                "Expected {} generic argument(s), but got {} instead",
                                fn_generics.params.len(),
                                def.0.len(),
                            ),
                        ).into_compile_error().into()
                    }
                }

                fn_name_expanded = args.listen.0
                    .map(|x| x.to_token_stream())
                    .unwrap_or(fn_name_expanded);
            }

            Ok(None) => {}
            Err(e) => return e.to_compile_error().into(),
        }

        if fn_name.to_string().as_str() != "__anonymous__" {
            let existed = !recorded_events.insert(fn_name.clone().to_string());
            if existed {
                return syn::Error::new_spanned(input, "Duplicate listening to the same TaskHookEvent")
                    .into_compile_error()
                    .into()
            }
        } else {
            let next = fn_generics.type_params().next();
            if next.is_some() || next.filter(|x| x.colon_token.is_none()).is_some() {
                return syn::Error::new_spanned(input, "__anonymous__ is a reserved name and cannot be used with narrow-based event generics")
                    .into_compile_error()
                    .into()
            }
        }

        if !matches!(fn_return, syn::ReturnType::Default) {
            return syn::Error::new_spanned(input, "Unexpected return parameter specified for method")
                .into_compile_error()
                .into()
        }

        if fn_sig.asyncness.is_none() {
            return syn::Error::new_spanned(fn_sig, ASYNC_REQUIRED_ERR)
                .into_compile_error()
                .into();
        }

        if fn_sig.abi.is_some() {
            return syn::Error::new_spanned(fn_sig, ABI_UNSUPPORTED_ERR)
                .into_compile_error()
                .into();
        }

        if fn_sig.unsafety.is_some() {
            *fn_block = quote! {
                unsafe { #fn_block }
            };
        }

        if item_defaults_enabled && matches!(macro_args, HookMacroArguments::Disabled) {
            return syn::Error::new_spanned(fn_sig, DEFAULT_NOT_ALLOWED_ERR)
                .into_compile_error()
                .into();
        }

        if item_defaults_enabled && !encountered_default {
            defaults.clear();
            encountered_default = true;
        }

        let (
            (ctx_name, ctx_type),
            (arg_names, arg_types)
        ) = match extract_arguments(&mut fn_sig.inputs) {
            Ok(res) => res,
            Err(e) => return e.to_compile_error().into(),
        };

        let impl_child_generics: Punctuated<_, Comma> = Punctuated::from_iter(
            fn_generics.params.iter()
                .filter(|x| {
                    if let syn::GenericParam::Type(ty) = x {
                        return ty.colon_token.is_some()
                    };

                    return true;
                })
                .chain(parent_generics.params.iter())
        );

        let mut event_expanded: TokenStream2 = quote! { #fn_name };

        if !fn_generics.params.is_empty() {
            let param_names = fn_generics.params
                .iter()
                .map(|param| match param {
                    syn::GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
                    syn::GenericParam::Type(pt) => pt.ident.clone(),
                    syn::GenericParam::Const(ct) => ct.ident.clone(),
                })
                .collect::<Punctuated<_, Comma>>();

            event_expanded = if fn_name.to_string().as_str() != "__anonymous__" {
                quote! { #event_expanded < #param_names > }
            } else { quote! { #param_names } }
        }

        if item_defaults_enabled || !encountered_default {
            match push_defaults(
                item_defaults_enabled,
                fn_name,
                item_defaults,
                &mut defaults,
                &event_expanded,
                &fn_generics.params
            ) {
                Ok(()) => {},
                Err(e) => return e.to_compile_error().into(),
            }
        }

        let mut payload_name = quote! { payload };
        let mut payload_extraction = Some(quote! { let (#arg_names): &(#arg_types) = payload; });

        if arg_names.len() == 0 {
            payload_name = quote! { _payload };
            payload_extraction = None;
        }

        let expanded = quote! {
            #[::async_trait::async_trait]
            impl <#impl_child_generics> ::chronographer::task::hooks::TaskHook<#event_expanded> for #taskhook_impl_end_expanded {
                async fn on_event(&self, #ctx_name: #ctx_type, #payload_name: &<#event_expanded as ::chronographer::task::hooks::TaskHookEvent>::Payload<'_>) {
                    #payload_extraction
                    #fn_block
                }
            }
        };

        expanded_listeners = quote! {
            #expanded_listeners
            #expanded
        };
    }

    let mut auto_attachment_expanded = None;
    if let HookMacroArguments::Enabled(auto_attach_fn_name) = macro_args {
        let mut default_attachments_expanded = quote! { };
        for default in defaults {
            default_attachments_expanded = quote! {
                #default_attachments_expanded
                hooks_layer.attach::<#default>(self.clone()).await;
            };
        }

        auto_attachment_expanded = Some(quote! {
            impl #parent_generics #taskhook_impl_end_expanded {
                pub async fn #auto_attach_fn_name(
                    self: std::sync::Arc<Self>,
                    hooks_layer: &impl ::chronographer::task::TaskHookLayer,
                ) {
                    #default_attachments_expanded
                }
            }
        })
    }

    quote! {
        #auto_attachment_expanded
        #expanded_listeners
    }.into()
}