use crate::workflow::utils::{ArgumentParser, ValueSource, WorkflowTransform};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitInt, parenthesized};

pub enum ThresholdReachBehavior {
    Error,
    Skip,
    Custom(syn::Expr),
}

impl Parse for ThresholdReachBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "error" => Ok(Self::Error),
            "skip" => Ok(Self::Skip),
            "custom" => {
                let content;
                parenthesized!(content in input);

                Ok(Self::Custom(content.parse()?))
            }

            _ => Err(input.error("Unknown threshold count behaviour")),
        }
    }
}

impl ToTokens for ThresholdReachBehavior {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            ThresholdReachBehavior::Error => {
                quote! { chronographer::task:::frames:thresholdframe::ThresholdSuccessReachBehaviour }
            }
            ThresholdReachBehavior::Skip => todo!(),
            ThresholdReachBehavior::Custom(expr) => quote! { #expr },
        };

        tokens.append_all(expanded);
    }
}

pub enum ThresholdCountBehavior {
    Identity,
    Successes,
    Failures,
    // ConsecutiveSuccesses,
    // ConsecutiveFailures,
    Custom(syn::Expr),
}

impl Parse for ThresholdCountBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "identity" => Ok(Self::Identity),
            "successes" => Ok(Self::Successes),
            "failures" => Ok(Self::Failures),
            // "consecutive_successes" => Ok(Self::ConsecutiveSuccesses),
            // "consecutive_failures" => Ok(Self::ConsecutiveFailures),
            "custom" => {
                let content;
                parenthesized!(content in input);

                Ok(Self::Custom(content.parse()?))
            }

            _ => Err(input.error("Unknown threshold count behaviour")),
        }
    }
}

impl ToTokens for ThresholdCountBehavior {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            ThresholdCountBehavior::Identity => {
                quote! { chronographer::task::frames::thresholdframe::ThresholdIdentityCountLogic }
            }
            ThresholdCountBehavior::Successes => {
                quote! { chronographer::task::frames::thresholdframe::ThresholdSuccessesCountLogic }
            }
            ThresholdCountBehavior::Failures => {
                quote! { chronographer::task::frames::thresholdframe::ThresholdErrorsCountLogic }
            }
            ThresholdCountBehavior::Custom(expr) => quote! { #expr },
        };

        tokens.append_all(expanded);
    }
}

pub struct ThresholdArguments {
    max: ValueSource<LitInt>,
    reach_behavior: Option<ThresholdReachBehavior>,
    count_behavior: Option<ThresholdCountBehavior>,
}

impl Parse for ThresholdArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let max = argument_parser.parse_required("max")?;
        let reach_behavior = argument_parser.parse_optional("reach_behavior")?;
        let count_behavior = argument_parser.parse_optional("count_behavior")?;
        Ok(ThresholdArguments {
            max,
            reach_behavior,
            count_behavior,
        })
    }
}

impl WorkflowTransform for ThresholdArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let max = &self.max;
        let expanded_reach_behavior = self
            .reach_behavior
            .as_ref()
            .map(|x| quote! { .reach_behaviour(#x) });

        let expanded_count_logic = self
            .count_behavior
            .as_ref()
            .map(|x| quote! { .count_behaviour(#x) });

        quote! {
            ::chronographer::task::frames::thresholdframe::ThresholdTaskFrame::builder()
                .frame(#toks)
                .threshold(std::num::NonZeroUsize::new(#max).unwrap())
                #expanded_reach_behavior
                #expanded_count_logic
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream2) -> TokenStream2 {
        quote! { ::chronographer::task::frames::thresholdframe::ThresholdTaskFrame::<#toks> }
    }
}
