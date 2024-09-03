use std::collections::HashSet;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use anyhow::{anyhow, bail, Result};

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
                    bail!("invalid delimiter for abstraction");
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

fn collect_inputs(tokens: TokenStream) -> Result<HashSet<String>> {
    let mut input_list: HashSet<String> = HashSet::new();
    for token in tokens {
        let input = match token {
            TokenTree::Ident(ident) => Ok(Some(ident.to_string())),
            TokenTree::Punct(punct_symbol) => match punct_symbol.as_char() {
                ',' => Ok(None),
                c => Err(anyhow!("invalid character '{c}' in input decorator")),
            },
            _ => Err(anyhow!("invalid input decorator")),
        }?;

        if let Some(t) = input {
            input_list.insert(t);
        }
    }
    Ok(input_list)
}

fn handle_io(tokens: &[TokenTree]) -> Result<(&[TokenTree], HashSet<String>)> {
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(symbol) => match symbol.as_char() {
                '@' => match &tokens[i + 1] {
                    TokenTree::Ident(raw) => {
                        let ident = raw.to_string();
                        if ident != "input" {
                            bail!("invalid decorator '{ident}'");
                        }
                        if let TokenTree::Group(g) = &tokens[i + 2] {
                            if g.delimiter() != Delimiter::Parenthesis {
                                bail!("invalid delimiter for input decorator");
                            }
                            return Ok((&tokens[i + 3..], collect_inputs(g.stream())?));
                        } else {
                            bail!("invalid decorator");
                        }
                    }
                    _ => bail!("invalid decorator"),
                },
                _ => {}
            },
            _ => {}
        }
        i += 1;
    }
    Ok((tokens, HashSet::new()))
}

#[proc_macro]
pub fn lambda(body: TokenStream) -> TokenStream {
    println!("body: {body:#?}");
    let tokens = body.into_iter().collect::<Vec<_>>();
    let (tokens, inputs) = handle_io(&tokens).unwrap();
    let expr = astize(&tokens).unwrap();
    let reduced = expr.eval();

    println!("inputs: {:?}", inputs);
    println!("{} --> {}", expr.code(), reduced.code());

    match construct_output(&reduced, &inputs).unwrap() {
        Some(output) => output,
        None => panic!(
            "expression reduced to '{}' which is not a valid output type",
            reduced.code()
        ),
    }
}

/// Convert variable-only applications to a nested tuple.
fn construct_output(expr: &Expr, valid_outputs: &HashSet<String>) -> Result<Option<TokenStream>> {
    match expr {
        Expr::Abstraction(_, _) => Ok(None),
        Expr::Application(e1, e2) => {
            let l_var = construct_output(e1, valid_outputs)?
                .and_then(|output| output.into_iter().next())
                .ok_or(anyhow!("Expected a valid output type for left-applicative"))?;

            let r_var = construct_output(e2, valid_outputs)?
                .and_then(|output| output.into_iter().next())
                .ok_or(anyhow!(
                    "Expected a valid output type for right-applicative"
                ))?;

            // Construct the tuple
            let tuple_inner = TokenStream::from_iter([
                l_var,
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                r_var,
            ]);
            let app = TokenTree::Group(Group::new(Delimiter::Parenthesis, tuple_inner));
            Ok(Some(TokenStream::from(app)))
        }
        Expr::Variable(Var { name, .. }) => {
            if valid_outputs.contains(name) {
                Ok(Some(TokenStream::from(TokenTree::Ident(Ident::new(
                    &name,
                    Span::call_site(),
                )))))
            } else {
                bail!("expression reduction contains '{name}', which is not decorated as an input");
            }
        }
    }
}
