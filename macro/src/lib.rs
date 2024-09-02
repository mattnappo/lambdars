extern crate proc_macro;
use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use anyhow::{bail, Result};
use quote::quote;

use lambdars_core::ast::{Expr, Var};

const LAMBDA_TOK: &str = "L";

fn astize(tokens: &[TokenTree]) -> Result<Expr> {
    let mut i = 0;
    let mut ast: Vec<Expr> = vec![];
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(raw) => {
                let ident = raw.to_string();
                // Build an abstraction
                if ident.starts_with(LAMBDA_TOK) {
                    let variable = &ident[1..];
                    let rhs = astize(&tokens[i + 2..])?;
                    let abs = Expr::abstraction(variable, rhs);
                    ast.push(abs);
                    i = tokens.len();
                } else {
                    ast.push(Expr::variable(ident));
                }
            }
            TokenTree::Group(group) => {
                if group.delimiter() != Delimiter::Parenthesis {
                    bail!("invalid delimiter");
                }
                let inner = astize(&group.stream().into_iter().collect::<Vec<_>>())?;
                ast.push(inner);
            }
            _ => {}
        }
        i += 1;
    }
    // Apply in left-most order
    let expr = ast
        .into_iter()
        .reduce(|e, n| Expr::application(e, n))
        .expect("empty expression");
    Ok(expr)
}

#[proc_macro]
pub fn lambda(body: TokenStream) -> TokenStream {
    println!("body: {body:#?}");
    let expr = astize(&body.into_iter().collect::<Vec<_>>()).unwrap();
    let reduced = expr.eval();
    println!("uhhlihh");

    println!("{} --> {}", expr.code(), reduced.code());

    let output = match reduced {
        Expr::Variable(Var { name, .. }) => {
            TokenStream::from(TokenTree::Ident(Ident::new(&name, Span::call_site())))
        }
        _ => TokenStream::new(),
    };
    println!("output: {output:#?}");
    output
}
