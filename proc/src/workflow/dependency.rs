use crate::workflow::utils::{ArgumentParser, WorkflowTransform};
use proc_macro2::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::{Parse, ParseStream};
use syn::{BinOp, UnOp};

// TODO: Add support for consecutive failures and consecutive successes
pub enum TaskRunMetric {
    Failure(syn::LitInt),
    Success(syn::LitInt),
    // ConsecutiveSuccesses(syn::LitInt),
    // ConsecutiveFailures(syn::LitInt),
    Any(syn::LitInt),
    Custom(Box<syn::Expr>),
}

pub enum AtomicDependency {
    Task {
        dependency: syn::Ident,
        value: TaskRunMetric,
    },

    Function(syn::Ident),
    Dynamic(Box<syn::ExprClosure>),
}

macro_rules! translate_task_run_metric {
    ($run_metric: expr, $ident: ident, $method_name: ident) => {
        if let TaskRunMetric::$ident(val) = &$run_metric {
            return (
                quote! { $method_name },
                quote! { std::num::NonZeroU16::new(#val).unwrap() },
            );
        }
    };
}

fn get_task_metric_vals(value: &TaskRunMetric) -> (TokenStream2, TokenStream2) {
    translate_task_run_metric!(value, Failure, failed_runs);
    translate_task_run_metric!(value, Success, successful_runs);
    translate_task_run_metric!(value, Any, runs);
    unreachable!()
}

impl ToTokens for AtomicDependency {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            AtomicDependency::Task { dependency, value } => {
                let (expanded_method_name, expanded_value) = get_task_metric_vals(value);
                quote! { chronographer::task::dependency::FrameDependency::#expanded_method_name(#dependency, #expanded_value).await }
            }

            AtomicDependency::Function(func) => {
                quote! { #func }
            }

            AtomicDependency::Dynamic(closure) => {
                quote! { #closure }
            }
        };

        tokens.append_all(expanded);
    }
}

macro_rules! task_run_metric {
    ($ident: expr, $assign_expr: expr, $lit: literal, $name: ident) => {{
        if $ident.to_string().as_str() == $lit {
            if let syn::Expr::Lit(literal) = $assign_expr.right.as_ref()
                && let syn::Lit::Int(value) = &literal.lit
            {
                return Ok(TaskRunMetric::$name(value.clone()));
            };

            return Err(syn::Error::new_spanned(
                &$assign_expr,
                "Expected an integer literal as value, but got something else",
            ));
        }
    }};
}

fn get_run_metric(ident: &syn::Ident, assign_expr: &syn::ExprAssign) -> syn::Result<TaskRunMetric> {
    task_run_metric!(ident, assign_expr, "any", Any);
    task_run_metric!(ident, assign_expr, "success", Success);
    task_run_metric!(ident, assign_expr, "failures", Failure);
    // task_run_metric!(ident, assign_expr, "consecutive_failures", ConsecutiveFailures);
    // task_run_metric!(ident, assign_expr, "consecutive_successes", ConsecutiveSuccesses);
    if ident.to_string().as_str() == "custom" {
        return Ok(TaskRunMetric::Custom(assign_expr.right.clone()));
    }

    Err(syn::Error::new_spanned(
        ident,
        "Expected either \"any\", \"successes\", \"failures\",  but got something else",
    ))
}

impl TryInto<AtomicDependency> for &syn::Expr {
    type Error = syn::Error;

    fn try_into(self) -> Result<AtomicDependency, Self::Error> {
        match self {
            syn::Expr::Call(expr_call) => {
                let syn::Expr::Path(expr_path) = expr_call.func.as_ref() else {
                    return Err(syn::Error::new_spanned(
                        &expr_call.func,
                        "Expected an identifier but got something else",
                    ));
                };

                let Some(path) = expr_path.path.segments.last() else {
                    return Err(syn::Error::new_spanned(
                        expr_path,
                        "Expected an identifier but got something else",
                    ));
                };

                match path.ident.to_string().as_str() {
                    "dynamic" => {
                        if expr_call.args.len() != 1 {
                            return Err(syn::Error::new_spanned(
                                path,
                                "Expected a closure for \"dynamic\" but got nothing or more than needed",
                            ));
                        }

                        let first = expr_call.args.first().unwrap();
                        let syn::Expr::Closure(expr_closure) = &first else {
                            return Err(syn::Error::new_spanned(
                                first,
                                "Expected a closure for the \"dynamic\", but got something else",
                            ));
                        };

                        Ok(AtomicDependency::Dynamic(Box::new(expr_closure.clone())))
                    }

                    _ => match expr_call.args.len() {
                        0 => Err(syn::Error::new_spanned(
                            expr_call,
                            "Invalid dependency expression, perhaps you meant to specify an argument?",
                        )),

                        1 => {
                            let first = expr_call.args.first().unwrap();
                            let syn::Expr::Assign(assign_expr) = first else {
                                return Err(syn::Error::new_spanned(
                                    first,
                                    "Expected a named argument but got something else",
                                ));
                            };

                            let syn::Expr::Path(expr_path) = assign_expr.left.as_ref() else {
                                return Err(syn::Error::new_spanned(
                                    &expr_call.func,
                                    "Expected an identifier but got something else",
                                ));
                            };

                            let Some(arg_path) = expr_path.path.segments.last() else {
                                return Err(syn::Error::new_spanned(
                                    expr_path,
                                    "Expected an identifier but got something else",
                                ));
                            };

                            let run_metric = get_run_metric(&arg_path.ident, assign_expr)?;
                            Ok(AtomicDependency::Task {
                                dependency: path.ident.clone(),
                                value: run_metric,
                            })
                        }
                        _ => Err(syn::Error::new_spanned(
                            path,
                            "Expected a single named argument for a Task or none for a function, got more",
                        )),
                    },
                }
            }

            syn::Expr::Path(expr_path) => {
                let last = expr_path.path.segments.last().unwrap();
                Ok(AtomicDependency::Function(last.ident.clone()))
            }

            value => Err(syn::Error::new_spanned(
                value,
                "Expected an identifier but got something else",
            )),
        }
    }
}

pub enum Dependency {
    Atomic(Box<AtomicDependency>),
    Or(Box<Dependency>, Box<Dependency>),
    And(Box<Dependency>, Box<Dependency>),
    Xor(Box<Dependency>, Box<Dependency>),
    Not(Box<Dependency>),
}

impl ToTokens for Dependency {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            Dependency::Atomic(dep) => return dep.to_tokens(tokens),
            Dependency::Or(dep1, dep2) => quote! { #dep1 | #dep2 },
            Dependency::And(dep1, dep2) => quote! { #dep1 & #dep2 },
            Dependency::Xor(dep1, dep2) => quote! { (#dep1 & !#dep2) | (!#dep1 & #dep2) },
            Dependency::Not(dep) => quote! { !#dep },
        };

        tokens.append_all(expanded);
    }
}

impl TryInto<Dependency> for &syn::Expr {
    type Error = syn::Error;

    fn try_into(self) -> Result<Dependency, Self::Error> {
        match self {
            syn::Expr::Paren(paren) => paren.expr.as_ref().try_into(),

            syn::Expr::Binary(bin) => match bin.op {
                BinOp::And(..) => Ok(Dependency::And(
                    Box::new(bin.left.as_ref().try_into()?),
                    Box::new(bin.right.as_ref().try_into()?),
                )),

                BinOp::Or(..) => Ok(Dependency::Or(
                    Box::new(bin.left.as_ref().try_into()?),
                    Box::new(bin.right.as_ref().try_into()?),
                )),

                BinOp::BitXor(..) => Ok(Dependency::Xor(
                    Box::new(bin.left.as_ref().try_into()?),
                    Box::new(bin.right.as_ref().try_into()?),
                )),

                _ => Err(syn::Error::new_spanned(
                    bin.op,
                    "Unknown boolean based operator",
                )),
            },

            syn::Expr::Unary(unary) => {
                if let UnOp::Not(..) = unary.op {
                    return Ok(Dependency::Not(Box::new(unary.expr.as_ref().try_into()?)));
                }

                Err(syn::Error::new_spanned(
                    unary.op,
                    "Unknown boolean based operator",
                ))
            }

            value => Ok(Dependency::Atomic(Box::new(value.try_into()?))),
        }
    }
}

impl Parse for Dependency {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        (&input.parse::<syn::Expr>()?).try_into()
    }
}

pub enum UnresolveBehavior {
    Fail,
    Skip,
    Custom(Box<syn::Expr>),
}

impl ToTokens for UnresolveBehavior {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expanded = match self {
            UnresolveBehavior::Fail => {
                quote! { chronographer::task::frames::dependencyframe::DependencyUnresolveFail::<_>::default() }
            }
            UnresolveBehavior::Skip => {
                quote! { chronographer::task::frames::dependencyframe::DependencyUnresolveSkip::<_>::default() }
            }
            UnresolveBehavior::Custom(expr) => quote! { #expr },
        };

        tokens.append_all(expanded);
    }
}

impl Parse for UnresolveBehavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Expr>()? {
            syn::Expr::Call(call) => {
                let syn::Expr::Path(expr_path) = call.func.as_ref() else {
                    return Err(input.error(
                        "Expected \"custom\" as a simple identifier but got something instead",
                    ));
                };

                if expr_path.path.segments.len() != 1 {
                    return Err(input.error(
                        "Expected \"custom\" as a simple identifier but got something instead",
                    ));
                }

                let mut argument_parser = ArgumentParser::new(input);
                let value = argument_parser.parse_required("value")?;
                Ok(Self::Custom(value))
            }

            syn::Expr::Path(expr_path) => {
                if expr_path.path.segments.len() != 1 {
                    return Err(input.error("Expected one ident but got multiple instead"));
                }

                let path = expr_path.path.segments.last().unwrap();
                match path.ident.to_string().as_str() {
                    "fail" => Ok(Self::Fail),
                    "skip" => Ok(Self::Skip),
                    _ => {
                        Err(input
                            .error("Expected either \"fail\", \"skip\" but got something else"))
                    }
                }
            }

            _ => Err(input.error("Unknown unresolve behavior")),
        }
    }
}

pub struct DependencyArguments {
    dep: Dependency,
    unresolve_behavior: Option<UnresolveBehavior>,
}

impl Parse for DependencyArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut argument_parser = ArgumentParser::new(input);
        let dep = argument_parser.parse_required("dep")?;
        let unresolve_behavior = argument_parser.parse_optional("unresolve")?;
        Ok(DependencyArguments {
            dep,
            unresolve_behavior,
        })
    }
}

impl WorkflowTransform for DependencyArguments {
    fn transform(&self, toks: TokenStream2) -> TokenStream2 {
        let dependency = &self.dep;
        let expanded_dependency = quote! { #dependency };
        let expanded_unresolve_behavior = self
            .unresolve_behavior
            .as_ref()
            .map(|x| quote! { .unresolve(#x) });

        quote! {
            ::chronographer::task::frames::dependencyframe::DependencyTaskFrame::builder()
                .frame(#toks)
                .dependency(#expanded_dependency)
                #expanded_unresolve_behavior
                .build()
        }
    }

    fn get_type(&self, toks: TokenStream) -> TokenStream {
        quote! { ::chronographer::task::frames::dependencyframe::DependencyTaskFrame::<#toks> }
    }
}
