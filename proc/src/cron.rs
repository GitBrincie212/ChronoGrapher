use chronographer_utils::{
    cron_lexer,
    cron_parser::{AstNode, AstTreeNode, CronParser},
    validator::validate_ast_node,
};
use proc_macro::TokenStream;
use quote::quote;


pub fn cron(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let tokens = match cron_lexer::tokenize_from_tokens(input2) {
        Ok(t) => t,
        Err((err, span)) => {
            return syn::Error::new(span, err.to_string())
                .to_compile_error()
                .into();
        }
    };

    let mut ast: [AstNode; 6] = std::array::from_fn(|_| AstNode::default());
    for (i, field_tokens) in tokens.iter().enumerate() {
        let mut parser = CronParser::new(field_tokens);
        ast[i] = match parser.parse_field() {
            Ok(node) => node,
            Err(e) => {
                let span = field_tokens
                    .first()
                    .and_then(|t| t.span)
                    .unwrap_or(proc_macro2::Span::call_site());
                return syn::Error::new(span, e.to_string())
                    .to_compile_error()
                    .into();
            }
        }
    }

    for (i, node) in ast.iter().enumerate() {
        if let Err(e) = validate_ast_node(node, i) {
            let span = tokens[i]
                .first()
                .and_then(|t| t.span)
                .unwrap_or(proc_macro2::Span::call_site());
            return syn::Error::new(span, e.to_string())
                .to_compile_error()
                .into();
        }
    }

    let fields: Vec<_> = ast.iter().map(ast_node_to_tokens).collect();
    quote! {
        chronographer::task::schedule::TaskScheduleCron::new([#(#fields),*])
    }
    .into()
}

fn ast_node_to_tokens(node: &AstNode) -> proc_macro2::TokenStream {
    match &node.kind {
        AstTreeNode::Wildcard => quote! { chronographer::task::schedule::CronField::Wildcard },
        AstTreeNode::Exact(v) => quote! { chronographer::task::schedule::CronField::Exact(#v) },
        AstTreeNode::Unspecified => {
            quote! { chronographer::task::schedule::CronField::Unspecified }
        }
        AstTreeNode::Range(s, e) => {
            let AstTreeNode::Exact(s_val) = &s.kind else {
                unreachable!()
            };
            let AstTreeNode::Exact(e_val) = &e.kind else {
                unreachable!()
            };
            quote! { chronographer::task::schedule::CronField::Range(#s_val, #e_val) }
        }

        AstTreeNode::Step(base, step) => {
            let base_tokens = ast_node_to_tokens(base);
            quote! { chronographer::task::schedule::CronField::Step(Box::new(#base_tokens), #step) }
        }
        AstTreeNode::List(items) => {
            let items: Vec<_> = items.iter().map(ast_node_to_tokens).collect();
            quote! { chronographer::task::schedule::CronField::List(vec![#(#items),*]) }
        }
        AstTreeNode::LastOf(None) => {
            quote! { chronographer::task::schedule::CronField::Last(None) }
        }
        AstTreeNode::LastOf(Some(v)) => {
            let v = *v as i8;
            quote! { chronographer::task::schedule::CronField::Last(Some(#v)) }
        }
        AstTreeNode::NearestWeekday(inner) => {
            let AstTreeNode::Exact(v) = &inner.kind else {
                unreachable!()
            };
            quote! { chronographer::task::schedule::CronField::NearestWeekday(#v) }
        }
        AstTreeNode::NthWeekday(a, b) => {
            quote! { chronographer::task::schedule::CronField::NthWeekday(#a, #b) }
        }
    }
}
