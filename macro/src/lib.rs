extern crate proc_macro;
use proc_macro::{Delimiter, TokenStream, TokenTree};

use anyhow::{bail, Result};

use lambdars_core::ast::Expr;

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
    println!("{}", expr.code());

    TokenStream::new()
}
