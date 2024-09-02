use std::collections::HashMap;

type Scope = HashMap<Var, u32>;

/// A variable used in a lambda expression.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Var {
    pub name: String,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// An expression consisting of just a single variable.
    Variable(Var),
    /// An expression of the form `lambda <var> . <expr>`.
    Abstraction(Var, Box<Expr>),
    /// An expression of the form `(M N)`.
    Application(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Helper method to construct a variable expression.
    pub fn variable(s: impl AsRef<str>) -> Self {
        Expr::Variable(Var::from(s))
    }

    /// Helper method to construct an abstraction expression.
    pub fn abstraction(s: impl AsRef<str>, e: Expr) -> Self {
        Expr::Abstraction(Var::from(s), Box::new(e))
    }

    /// Helper method to construct an application expression.
    pub fn application(e1: Expr, e2: Expr) -> Self {
        Expr::Application(Box::new(e1), Box::new(e2))
    }

    /// Print an expression as a string in this language.
    pub fn code(&self) -> String {
        use Expr::*;
        match self {
            Variable(v) => v.code(),
            Abstraction(v, e) => format!("(\\{}. {})", v.code(), &*e.code()),
            Application(e1, e2) => format!("({} {})", &*e1.code(), &*e2.code()),
        }
    }

    /// Reduce a lambda expression.
    pub fn eval(&self) -> Expr {
        self.canonicalize().eval_inner()
    }
}

impl Expr {
    fn sub(&self, var: &Var, e: Expr) -> Expr {
        use Expr::*;
        match self {
            ev @ Variable(v) => {
                match (var.ident, v.ident) {
                    // (Some(i), Some(j)) if i == j && var.name == v.name => e.clone(),
                    (Some(i), Some(j)) if i == j => e.clone(),
                    _ => ev.clone(),
                }
            }
            Abstraction(v, f) => Abstraction(v.clone(), Box::new(f.sub(var, e.clone()))),
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
    fn eval_inner(&self) -> Expr {
        use Expr::*;
        match self.clone() {
            Application(e1, e2) => {
                let (e1, e2) = (Box::new(e1.eval_inner()), Box::new(e2.eval_inner()));
                match *e1.clone() {
                    Abstraction(var, e) => e.sub(&var, *e2.clone()).eval_inner(),
                    _ => Application(e1, e2),
                }
            }
            other => other,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Helper for testing.
    pub fn eval(e: &Expr) -> Expr {
        let reduced = e.eval();
        println!("{} --> {}", e.code(), reduced.code());
        println!("reduced: {:#?}", reduced);
        reduced
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

        // [ \x. x (\x. x) (\x. x x) ] a
        // -> a (\x.x) (\x. x x)
        // -> a (\x. x x)
        let apply = Expr::application(outer_abstraction.clone(), Expr::variable("a"));
        eval(&apply);
        {
            let id = Expr::abstraction("x", Expr::variable("x"));
            let copy = Expr::abstraction(
                "y",
                Expr::application(Expr::variable("y"), Expr::variable("y")),
            );
            let f = Expr::abstraction(
                "x",
                Expr::application(Expr::variable("x"), Expr::application(id, copy)),
            );
            let g = Expr::application(f, Expr::variable("a"));
            eval(&g);
        }
    }

    #[test]
    fn test_eval3() {
        let id1 = Expr::abstraction("x", Expr::variable("x"));
        let id2 = Expr::abstraction("y", Expr::abstraction("x", Expr::variable("x")));
        let f = Expr::application(id1, id2);
        eval(&f);
    }

    #[test]
    fn test_eval4() {
        // (x \y. (\x.x))
        let id1 = Expr::variable("x");
        let id2 = Expr::abstraction("y", Expr::abstraction("x", Expr::variable("x")));
        let id3 = Expr::application(id2, Expr::variable("a"));
        let f = Expr::application(id1, id3);
        eval(&f);
    }

    #[test]
    fn test_eval5() {
        // (x \y. (\x.x))
        let id1 = Expr::variable("x");
        let id2 = Expr::abstraction("y", Expr::abstraction("x", Expr::variable("x")));
        let id3 = Expr::application(id2, Expr::variable("a"));
        let f = Expr::application(id1, id3);
        eval(&f);
    }

    #[test]
    fn test_complex() {
        // ((\f.(\x.f (\y.(\z.y (z z)) (\w.x (w w))))) (\p.(\q.p (q q)))) (\a.\b.a)
        let inner_abstraction1 = Expr::abstraction("w", Expr::variable("w"));
        let middle_abstraction = Expr::abstraction("z", inner_abstraction1.clone());
        let inner_application = Expr::application(
            Expr::variable("f"),
            Expr::application(Expr::variable("y"), Expr::variable("y")),
        );
        let inner_abstraction = Expr::abstraction("x", inner_application);
        let application1 = Expr::application(inner_abstraction, middle_abstraction.clone());
        let body = Expr::application(application1, Expr::variable("f"));
        let outer_abstraction = Expr::abstraction("f", body);
        let final_application = Expr::application(outer_abstraction, middle_abstraction);

        let output = Expr::application(final_application, Expr::variable("t"));
        eval(&output);
    }
}
