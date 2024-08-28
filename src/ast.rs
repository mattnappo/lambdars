use std::collections::HashMap;

type Scope = HashMap<Var, u32>;

/// A variable used in a lambda expression.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Var {
    name: String,
    ident: Option<u32>,
}

impl Var {
    fn from(s: impl AsRef<str>) -> Self {
        Var {
            name: s.as_ref().to_string(),
            ident: None,
        }
    }

    fn with_ident(&self, label: u32) -> Self {
        Var {
            name: self.name.clone(),
            ident: Some(label),
        }
    }

    fn code(&self) -> String {
        match self.ident {
            Some(i) => format!("{}{i}", self.name),
            None => format!("{}", self.name),
        }
    }
}

/// An untyped lambda expression.
#[derive(Debug, Clone)]
enum Expr {
    /// An expression consisting of just a single variable.
    Variable(Var),
    /// An expression of the form `lambda <var> . <expr>`.
    Abstraction(Var, Box<Expr>),
    /// An expression of the form `(M N)`.
    Application(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Helper method to construct a variable expression.
    fn variable(s: impl AsRef<str>) -> Self {
        Expr::Variable(Var::from(s))
    }

    /// Helper method to construct an abstraction expression.
    fn abstraction(s: impl AsRef<str>, e: Expr) -> Self {
        Expr::Abstraction(Var::from(s), Box::new(e))
    }

    /// Helper method to construct an application expression.
    fn application(e1: Expr, e2: Expr) -> Self {
        Expr::Application(Box::new(e1), Box::new(e2))
    }

    /// Print an expression as a string in this language.
    fn code(&self) -> String {
        use Expr::*;
        match self {
            Variable(v) => v.code(),
            Abstraction(v, e) => format!("(\\{}. {})", v.code(), &*e.code()),
            Application(e1, e2) => format!("({} {})", &*e1.code(), &*e2.code()),
        }
    }

    /// Naively substitute all occurrences of `var` with `e` in `self`.
    fn sub(&self, var: &Var, e: Expr) -> Expr {
        use Expr::*;
        match self {
            ev @ Variable(Var { name: v, .. }) => {
                if *v == var.name {
                    e.clone()
                } else {
                    ev.clone()
                }
            }
            Abstraction(svar, f) => {
                // TODO: handle case where svar == var?
                Abstraction(svar.clone(), Box::new(f.sub(var, e.clone())))
            }
            Application(e1, e2) => Application(
                Box::new(e1.sub(var, e.clone())),
                Box::new(e2.sub(var, e.clone())),
            ),
        }
    }

    fn canonicalize_inner(&self, scope: &Scope, d: u32) -> Expr {
        use Expr::*;

        match self {
            Abstraction(v, e) => {
                // Enter a deeper scope
                let mut new_scope = scope.clone();
                new_scope.insert(v.clone(), d + 1);

                let ec = e.canonicalize_inner(&new_scope, d + 1);
                Abstraction(v.with_ident(d + 1), Box::new(ec))
            }
            Application(e1, e2) => {
                let e1c = e1.canonicalize_inner(scope, d);
                let e2c = e2.canonicalize_inner(scope, d);
                Application(Box::new(e1c), Box::new(e2c))
            }
            Variable(var) => {
                let lookup = scope
                    .get(&var)
                    .map(|t| var.with_ident(*t))
                    .unwrap_or_else(|| var.clone());

                Variable(lookup)
            }
        }
    }

    /// Canonicalize bound variables to avoid binding issues.
    fn canonicalize(&self) -> Expr {
        self.canonicalize_inner(&HashMap::new(), 0)
    }

    /// Evaluate an expression by performing beta-reduction and alpha-renaming
    /// when necessary.
    fn eval(&self) -> Expr {
        use Expr::*;

        match self.canonicalize() {
            // Can only reduce an application (M N) if M is an abstraction.
            Application(e1, e2) => match *e1 {
                Abstraction(var, e) => e.sub(&var, *e2),
                e => e,
            },
            // Variables and Abstractions cannot be reduced further.
            e => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper for testing.
    fn eval(e: &Expr) {
        println!("{} --> {}", e.code(), e.eval().code());
    }

    #[test]
    fn test_expr1() {
        // \x.x
        let id_fn = Expr::abstraction("x", Expr::variable("x"));

        // (\x.x) y
        let apply_y = Expr::application(id_fn, Expr::variable("y"));

        dbg!(&apply_y);
        println!("{}", apply_y.code());
    }

    #[test]
    fn test_expr2() {
        let part = Expr::abstraction("x", Expr::variable("y"));

        // \x. \y. x
        let true_fn = Expr::abstraction("x", part);
        eval(&true_fn);

        // (\x. (\y. x)) a
        let apply_true = Expr::application(true_fn, Expr::variable("a"));
        eval(&apply_true);

        dbg!(&apply_true);
        println!("{}", apply_true.code());
    }

    #[test]
    fn test_canonicalize() {
        // (\x.x) y
        let id_fn = Expr::abstraction("x", Expr::variable("x"));
        let apply_y = Expr::application(id_fn.clone(), Expr::variable("y"));

        let c = apply_y.canonicalize();
        println!("{}", c.code());

        // (\x. (\x. x) x) x
        let x = Expr::variable("x");

        let f = Expr::application(
            Expr::abstraction("x", Expr::application(id_fn, x.clone())),
            x,
        );
        println!("{}", f.code());

        let c = f.canonicalize();
        println!("{}", c.code());
    }

    #[test]
    fn test_eval1() {
        // (\x.x) y
        let id_fn = Expr::abstraction("x", Expr::variable("x"));
        let apply_y = Expr::application(id_fn, Expr::variable("y"));
        eval(&apply_y);
    }

    #[test]
    fn test_eval2() {
        // \x. x (\x. x) (\x. x x)
        let inner_abstraction1 = Expr::abstraction("x", Expr::variable("x"));
        let inner_abstraction2 = Expr::abstraction(
            "x",
            Expr::application(Expr::variable("x"), Expr::variable("x")),
        );
        let application1 = Expr::application(inner_abstraction1, inner_abstraction2);
        let body = Expr::application(Expr::variable("x"), application1);
        let outer_abstraction = Expr::abstraction("x", body);
        eval(&outer_abstraction);
    }
}
