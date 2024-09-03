use std::collections::HashSet;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use anyhow::{anyhow, bail, Result};

use lambdars_core::ast::{Expr, Var};

const LAMBDA_TOK: &str = "L";

fn astize(tokens: &[TokenTree]) -> Result<Expr> {
    let mut ast: Vec<Expr> = vec![];
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(raw) => {
                let ident = raw.to_string();
                if ident.starts_with(LAMBDA_TOK) {
                    let variable = &ident[1..];
                    // Collect tokens for the right-hand side
                    let rhs_tokens: Vec<_> = iter.by_ref().cloned().collect();
                    let rhs = astize(&rhs_tokens)?;
                    let abs = Expr::abstraction(variable, rhs);
                    ast.push(abs);
                } else {
                    ast.push(Expr::variable(ident));
                }
            }
            TokenTree::Group(group) => {
                if group.delimiter() != Delimiter::Parenthesis {
                    bail!("invalid delimiter for abstraction");
                }
                let inner_tokens: Vec<_> = group.stream().into_iter().collect();
                let inner = astize(&inner_tokens)?;
                ast.push(inner);
            }
            _ => {}
        }
    }

    // Apply in left-most order
    let expr = ast
        .into_iter()
        .reduce(|e, n| Expr::application(e, n))
        .expect("empty expression");
    Ok(expr)
}

fn collect_inputs(tokens: TokenStream) -> Result<HashSet<String>> {
    let mut inputs = HashSet::new();
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => {
                inputs.insert(ident.to_string());
            }
            TokenTree::Punct(punct_symbol) => {
                if punct_symbol.as_char() != ',' {
                    return Err(anyhow!(
                        "invalid character '{}' in input decorator",
                        punct_symbol.as_char()
                    ));
                }
            }
            _ => return Err(anyhow!("invalid input decorator")),
        }
    }
    Ok(inputs)
}

fn handle_io(tokens: &[TokenTree]) -> Result<(&[TokenTree], HashSet<String>)> {
    // Iterate with index and pattern match
    for (i, token) in tokens.iter().enumerate() {
        if let TokenTree::Punct(symbol) = token {
            if symbol.as_char() == '@' {
                if let Some(TokenTree::Ident(raw)) = tokens.get(i + 1) {
                    let ident = raw.to_string();
                    if ident != "input" {
                        bail!("invalid decorator '{ident}'");
                    }
                    if let Some(TokenTree::Group(group)) = tokens.get(i + 2) {
                        if group.delimiter() != Delimiter::Parenthesis {
                            bail!("invalid delimiter for input decorator");
                        }
                        // Collect inputs from the group stream
                        let inputs = collect_inputs(group.stream())?;
                        return Ok((&tokens[i + 3..], inputs));
                    } else {
                        bail!("invalid decorator");
                    }
                } else {
                    bail!("invalid decorator");
                }
            }
        }
    }
    Ok((tokens, HashSet::new()))
}

#[proc_macro]
pub fn lambda(body: TokenStream) -> TokenStream {
    let tokens = body.into_iter().collect::<Vec<_>>();
    let (tokens, inputs) = handle_io(&tokens).unwrap();
    let expr = astize(&tokens).unwrap();
    let reduced = expr.eval();

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
            let l_var = extract_valid_output(e1, valid_outputs)?;
            let r_var = extract_valid_output(e2, valid_outputs)?;

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
                let ident = TokenTree::Ident(Ident::new(name, Span::call_site()));
                Ok(Some(TokenStream::from(ident)))
            } else {
                bail!("expression reduction contains '{name}', which is not decorated as an input")
            }
        }
    }
}

// Helper function to extract valid output or bail if not present.
fn extract_valid_output(expr: &Expr, valid_outputs: &HashSet<String>) -> Result<TokenTree> {
    construct_output(expr, valid_outputs)?
        .ok_or(anyhow!("expected a valid output type"))?
        .into_iter()
        .next()
        .ok_or(anyhow!("expected a valid output type"))
}
