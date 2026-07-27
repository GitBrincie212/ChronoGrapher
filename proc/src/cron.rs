use chronographer_utils::cron_lexer::{Token, TokenType};
use chronographer_utils::errors::CronExpressionLexerErrors;
use chronographer_utils::{
    cron_parser::{AstNode, AstTreeNode, CronParser},
    validator::validate_ast_node,
};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::quote;

pub fn cron(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let tokens = match tokenize_from_tokens(input2) {
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
        chronographer::task::schedule::TaskScheduleCron::new([#(#fields),*, chronographer::task::schedule::CronField::Wildcard])
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

/// Maps a day-of-week / month identifier (e.g. `MON`, `dec`) to its numeric token type.
///
/// Matching is case-insensitive, so `MON`, `mon` and `MoN` are all accepted.
fn ident_to_token_type(ident: &Ident) -> Option<TokenType> {
    let token_type = match ident_upper(ident).as_str() {
        "L" => TokenType::Last,
        "W" => TokenType::NearestWeekday,
        "SUN" => TokenType::Value(1),
        "MON" => TokenType::Value(2),
        "TUE" => TokenType::Value(3),
        "WED" => TokenType::Value(4),
        "THU" => TokenType::Value(5),
        "FRI" => TokenType::Value(6),
        "SAT" => TokenType::Value(7),
        "JAN" => TokenType::Value(1),
        "FEB" => TokenType::Value(2),
        "MAR" => TokenType::Value(3),
        "APR" => TokenType::Value(4),
        "MAY" => TokenType::Value(5),
        "JUN" => TokenType::Value(6),
        "JUL" => TokenType::Value(7),
        "AUG" => TokenType::Value(8),
        "SEP" => TokenType::Value(9),
        "OCT" => TokenType::Value(10),
        "NOV" => TokenType::Value(11),
        "DEC" => TokenType::Value(12),
        _ => return None,
    };
    Some(token_type)
}

/// Uppercases an identifier's text once, avoiding the repeated `to_string().to_uppercase()`
/// allocations the field-boundary logic would otherwise incur per token.
fn ident_upper(ident: &Ident) -> String {
    let mut s = ident.to_string();
    s.make_ascii_uppercase();
    s
}

pub fn tokenize_from_tokens(
    input: proc_macro2::TokenStream,
) -> Result<[Vec<Token>; 6], (CronExpressionLexerErrors, proc_macro2::Span)> {
    let mut tokens: [Vec<Token>; 6] = std::array::from_fn(|_| Vec::new());
    let mut field_pos: usize = 0;

    #[derive(Clone, Copy, PartialEq)]
    enum Prev {
        None,
        Operator,
        Value,
        NumLit,
        IdentL,
    }
    let mut prev = Prev::None;

    for tt in input {
        // Uppercase identifier text once and reuse it for the boundary heuristic,
        // the `prev` bookkeeping and the final token mapping below.
        let ident_text = match &tt {
            TokenTree::Ident(id) => Some(ident_upper(id)),
            _ => None,
        };

        let is_value = match &tt {
            TokenTree::Literal(_) | TokenTree::Ident(_) => true,
            TokenTree::Punct(p) => matches!(p.as_char(), '*' | '?'),
            TokenTree::Group(_) => false,
        };

        if is_value && field_pos < 5 {
            let advance = match prev {
                Prev::None | Prev::Operator => false,
                Prev::NumLit => match &ident_text {
                    Some(s) => s != "L" && s != "W",
                    None => true,
                },
                Prev::IdentL => match &ident_text {
                    Some(s) => s != "W",
                    None => true,
                },
                Prev::Value => true,
            };
            if advance {
                field_pos += 1;
            }
        }

        prev = match (&tt, &ident_text) {
            (TokenTree::Literal(_), _) => Prev::NumLit,
            (TokenTree::Ident(_), Some(s)) if s == "L" => Prev::IdentL,
            (TokenTree::Ident(_), _) => Prev::Value,
            (TokenTree::Punct(p), _) if matches!(p.as_char(), '*' | '?') => Prev::Value,
            (TokenTree::Punct(_), _) => Prev::Operator,
            (TokenTree::Group(_), _) => prev,
        };

        match tt {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                let val: u32 = s
                    .parse()
                    .map_err(|_| (CronExpressionLexerErrors::UnknownCharacter, lit.span()))?;
                tokens[field_pos].push(Token {
                    start: 0,
                    token_type: TokenType::Value(val),
                    span: Some(lit.span()),
                });
            }
            TokenTree::Ident(ident) => {
                let token_type = ident_to_token_type(&ident)
                    .ok_or((CronExpressionLexerErrors::UnknownCharacter, ident.span()))?;
                tokens[field_pos].push(Token {
                    start: 0,
                    token_type,
                    span: Some(ident.span()),
                });
            }
            TokenTree::Punct(punct) => {
                let token_type = match punct.as_char() {
                    '*' => TokenType::Wildcard,
                    '-' => TokenType::Minus,
                    ',' => TokenType::ListSeparator,
                    '/' => TokenType::Step,
                    '#' => TokenType::NthWeekday,
                    '?' => TokenType::Unspecified,
                    _ => return Err((CronExpressionLexerErrors::UnknownCharacter, punct.span())),
                };
                tokens[field_pos].push(Token {
                    start: 0,
                    token_type,
                    span: Some(punct.span()),
                });
            }
            TokenTree::Group(_) => {}
        }
    }

    Ok(tokens)
}
