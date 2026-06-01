use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::parse::discouraged::Speculative;
use syn::Token;

pub enum ValueSource<T: Parse> {
    Lit(T),
    Function(syn::Ident),
    Closure(syn::ExprClosure),
    Macro(syn::ExprMacro),
}

impl<T: Parse> Parse for ValueSource<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();

        if fork.peek(syn::Ident) && fork.peek2(Token![!]) {
            let mac: syn::ExprMacro = input.parse()?;
            return Ok(Self::Macro(mac));
        }

        if fork.peek(Token![|]) {
            let closure: syn::ExprClosure = input.parse()?;
            return Ok(Self::Closure(closure));
        }

        if fork.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            return Ok(Self::Function(ident));
        }

        if let Ok(content) = fork.parse::<T>() {
            input.advance_to(&fork);
            return Ok(Self::Lit(content));
        }

        Err(input.error("Invalid expression"))
    }
}

impl<T: Parse + ToTokens> ToTokens for ValueSource<T> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let toks = match self {
            ValueSource::Lit(lit) => quote! { #lit },
            ValueSource::Function(ident) => quote! { #ident },
            ValueSource::Closure(closure) => quote! { #closure },
            ValueSource::Macro(macro_call) => quote! { #macro_call },
        };

        tokens.append_all(toks);
    }
}

pub enum MacroArgument<T: Parse> {
    Positional(T),
    Named {
        name: syn::Ident,
        value: T
    }
}

impl<T: Parse> MacroArgument<T> {
    pub fn name(&self) -> Option<&syn::Ident> {
        match self {
            MacroArgument::Positional(_) => None,
            MacroArgument::Named { name, ..} => Some(name)
        }
    }

    pub fn into_value(self) -> T {
        match self {
            MacroArgument::Positional(v) => v,
            MacroArgument::Named { value, ..} => value
        }
    }
}

pub struct ArgumentParser<'a> {
    input: ParseStream<'a>,
    expect_named: bool
}

impl<'a> ArgumentParser<'a> {
    pub fn new(input: ParseStream<'a>) -> Self {
        Self { input, expect_named: false }
    }

    fn try_consume_comma(&mut self) {
        if self.input.peek(Token![,]) {
            let _ = self.input.parse::<Token![,]>();
        }
    }

    fn parse_next<T: Parse>(&mut self) -> syn::Result<Option<MacroArgument<T>>> {
        if self.input.is_empty() {
            return Ok(None);
        }

        if self.input.peek(syn::Ident) && self.input.peek2(Token![=]) {
            let name = self.input.parse()?;
            self.input.parse::<Token![=]>()?;
            let value = self.input.parse()?;
            self.try_consume_comma();
            self.expect_named = true;

            return Ok(Some(MacroArgument::Named { name, value }));
        }

        if self.expect_named {
            return Err(self.input.error("Expected a named argument but got a positional instead"));
        }

        let value = self.input.parse()?;
        self.try_consume_comma();
        Ok(Some(MacroArgument::Positional(value)))
    }

    pub fn parse_required<T: Parse>(&mut self, expected: &'static str) -> syn::Result<T> {
        self.parse_next()?
            .ok_or_else(|| {
                syn::Error::new(self.input.span(), format!("Expected {expected} argument but got nothing"))
            })
            .and_then(|arg| {
                match arg.name() {
                    None => Ok(arg.into_value()),
                    Some(name) if name.to_string() == expected => Ok(arg.into_value()),
                    Some(unexpected) => Err(syn::Error::new(
                        self.input.span(),
                        format!("Expected {expected} argument but got {unexpected}")
                    ))
                }
            })
    }

    pub fn parse_optional<T: Parse>(&mut self, expected: &'static str) -> syn::Result<Option<T>> {
        Ok(self.parse_next()?
            .and_then(|x| {
                match x.name() {
                    Some(actual) if actual.to_string().as_str() == expected => Some(x.into_value()),
                    _ => None
                }
            })
        )
    }

    pub fn parse_remaining<T: Parse>(&mut self) -> syn::Result<Vec<T>> {
        let mut values = Vec::new();
        while let Some(parsed) = self.parse_next()? {
            values.push(parsed.into_value());
        }

        Ok(values)
    }
}

pub trait WorkflowTransform {
    fn transform(&self, toks: TokenStream2) -> TokenStream2;
    fn get_type(&self, toks: TokenStream2) -> TokenStream2;
}