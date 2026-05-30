use syn::parenthesized;
use syn::parse::{Parse, ParseStream};
use crate::workflow::utils::ArgumentParser;

pub enum ConditionReturnBehavior {
    Error,
    Success,
    Custom(syn::Expr)
}

impl Parse for ConditionReturnBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "error" => Ok(Self::Error),
            "success" => Ok(Self::Success),
            "custom" => {
                let content;
                parenthesized!(content in input);

                Ok(Self::Custom(content.parse()?))
            }

            _ => Err(input.error("Unknown condition return behaviour"))
        }
    }
}

pub struct ConditionArguments {
    predicate: syn::Ident,
    secondary: Option<syn::Ident>,
    return_with: Option<ConditionReturnBehavior>,
}

impl Parse for ConditionArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let predicate = argument_parser.parse_required("predicate")?;
        let secondary = argument_parser.parse_optional("secondary")?;
        let return_with = argument_parser.parse_optional("return_with")?;
        Ok(ConditionArguments { predicate, secondary, return_with })
    }
}