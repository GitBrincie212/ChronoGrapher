use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::{bracketed, Attribute, Token};
use syn::token::Comma;

#[derive(Debug)]
pub struct HookItemDefaultField(pub(crate) Punctuated<syn::Type, Token![,]>);

impl ToTokens for HookItemDefaultField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

impl Parse for HookItemDefaultField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();

        if let Ok(_) = fork.parse::<Token![<]>() {
            let args = Punctuated::<syn::Type, Comma>::parse_separated_nonempty(&fork)?;
            let _ = fork.parse::<Token![>]>()?;

            input.advance_to(&fork);
            return Ok(Self(args));
        }

        let mut args = Punctuated::new();
        args.push_value(input.parse()?);

        Ok(Self(args))
    }
}

/*
    Quite awkward code for parsing the #[hooks(...)] macro annotation for functions. Might fix this
    in the future, but for mostly prototype reasons I'm keeping it even if it's imperfect.
*/

// Syntax            | Enabled?   Provided Values
// ==================+===========================
// !default          | false,      Vec[]
// default = [...]   | true,       Vec[...]
// default           | true,       Vec[]
// <Unspecified>     | true,       Vec[]
#[derive(Debug)]
pub struct HookItemDefaults(pub(crate) bool, pub(crate) Punctuated<HookItemDefaultField, Comma>);

impl Parse for HookItemDefaults {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(HookItemDefaults(true, Punctuated::new()));
        }

        let mut negate = false;
        if input.peek(Token![!]) {
            let _ = input.parse::<Token![!]>();
            negate = true;
        }

        let ident = input.parse::<syn::Ident>()?;
        if ident.to_string().as_str() != "default" {
            return Err(input.error("Unknown ident, did you mean to use \"default\" instead?"))
        }

        if negate && !input.is_empty() {
            return Err(input.error("Unexpected token sequence after \"!default\""))
        } else if negate && input.is_empty() {
            return Ok(HookItemDefaults(false, Punctuated::new()));
        } else if input.is_empty() {
            return Ok(HookItemDefaults(true, Punctuated::new()));
        }

        let _ = input.parse::<Token![=]>()?;
        let content;
        bracketed!(content in input);

        let content = Punctuated::<HookItemDefaultField, Comma>::parse_separated_nonempty(&content)?;
        Ok(HookItemDefaults(true, content))
    }
}

#[derive(Debug)]
pub struct HookListen(pub(crate) Option<syn::Type>);

impl Parse for HookListen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self(None));
        }

        let parsed_ident = input.parse::<syn::Ident>()?;
        if parsed_ident.to_string().as_str() != "listen" {
            return Err(input.error("Unknown token sequence, did you mean to specify the \"listen\" parameter?"))
        }

        let _ = input.parse::<Token![=]>()?;
        let ty = input.parse::<syn::Type>()?;

        Ok(Self(Some(ty)))
    }
}

#[derive(Debug)]
pub struct HookAnnotationMacroArguments {
    pub(crate) defaults: HookItemDefaults,
    pub(crate) listen: HookListen
}

impl Parse for HookAnnotationMacroArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("Must specify either the \"default\" or \"listen\" parameter, or alternatively remove the annotation to specify none"))
        }

        let mut listen = None;
        let mut defaults = None;
        let mut first_loop = true;

        while !input.is_empty() {
            if !input.is_empty() && !first_loop {
                input.parse::<Token![,]>()?;
            }

            first_loop = false;
            let fork = input.fork();
            if let Ok(hook_defaults ) = fork.parse::<HookItemDefaults>() {
                if defaults.is_some() {
                    return Err(input.error("Already specified the \"default\" parameter, did you mean to use \"listen\"?"))
                }

                defaults = Some(hook_defaults);
                input.advance_to(&fork);
                continue;
            }

            let hook_listen = input.parse::<HookListen>()?;
            if listen.is_some() {
                return Err(input.error("Already specified the \"listen\" parameter, did you mean to use \"default\"?"))
            }

            listen = Some(hook_listen);
        }

        if listen.is_none() && defaults.is_none() {
            return Err(input.error("Must specify either the \"default\" or \"listen\" parameter, or alternatively remove the annotation to specify none"))
        }

        Ok(Self {
            defaults: defaults.unwrap_or(HookItemDefaults(true, Punctuated::new())),
            listen: listen.unwrap_or(HookListen(None))
        })
    }
}

impl HookAnnotationMacroArguments {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut result: Option<_> = None;
        for attr in attrs {
            let Some(path) = attr.path().segments.last() else {
                continue;
            };

            if path.ident.to_string() != "hook" {
                continue;
            }

            if result.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Cannot use the hook macro twice on the same item",
                ));
            }

            let list = attr.meta
                .require_list()?
                .tokens
                .clone();

            result = Some(HookAnnotationMacroArguments::parse.parse2(list)?);
        }

        Ok(result)
    }
}