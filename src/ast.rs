use std::collections::HashMap;

/// A variable used in a lambda expression.
#[derive(Debug, Clone)]
struct Var {
    name: String,
}

impl Var {
    fn from(s: impl AsRef<str>) -> Self {
        Var {
            name: s.as_ref().to_string(),
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
    // TODO: can put use Self::* here?

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
        match self {
            Expr::Variable(Var { name: v }) => v.to_string(),
            Expr::Abstraction(Var { name: v }, e) => format!("(\\{}. {})", v, &*e.code()),
            Expr::Application(e1, e2) => format!("({} {})", &*e1.code(), &*e2.code()),
        }
    }

    /// Substitute all occurrences of `var` with `e` in `self`.
    fn sub(self, var: &Var, e: Expr) -> Expr {
        todo!()
        //match self {}
    }

    // fn alpha_rename(self, )

    fn canonicalize_inner(&self, d: u32) -> (Expr, HashMap<u32, Var>) {
        use Expr::*;

        let mut m: HashMap<u32, Var> = HashMap::new();

        // TODO: turn this into a reduce with HashMap::extend
        let e = match self {
            Abstraction(v, e) => {
                m.insert(d, v.clone());
                let new_name = d.to_string();
                let (ec, m1) = e.canonicalize_inner(d);
                m.extend(m1);
                Abstraction(Var::from(new_name), Box::new(ec))
            }
            Application(e1, e2) => {
                let (e1c, m1) = e1.canonicalize_inner(d + 1);
                let (e2c, m2) = e2.canonicalize_inner(d + 1);
                m.extend(m1);
                m.extend(m2);
                Application(Box::new(e1c), Box::new(e2c))
            }
            variable => variable.clone(),
        };

        (e, m)
    }

    /// Canonicalize bound variables to avoid binding issues.
    fn canonicalize(&self) -> (Expr, HashMap<u32, Var>) {
        self.canonicalize_inner(0)
    }

    /// Evaluate an expression by performing beta-reduction and alpha-renaming
    /// when necessary.
    // TODO: make this take a &self, not a self. Same with `sub`.
    fn eval(self) -> Expr {
        use Expr::*;

        match self {
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

        // (\x. (\y. x)) a
        let apply_true = Expr::application(true_fn, Expr::variable("a"));

        dbg!(&apply_true);
        println!("{}", apply_true.code());
    }

    #[test]
    fn test_canonicalize() {
        // (\x.x) y
        let id_fn = Expr::abstraction("x", Expr::variable("x"));
        let apply_y = Expr::application(id_fn.clone(), Expr::variable("y"));

        let (c, m) = apply_y.canonicalize();
        println!("{:#?}", m);
        println!("{}", c.code());

        // (\x. (\x. x) x) x
        let x = Expr::variable("x");

        let f = Expr::application(
            Expr::abstraction("x", Expr::application(id_fn, x.clone())),
            x,
        );
        println!("{}", f.code());

        let (c, m) = f.canonicalize();
        println!("{:#?}", m);
        println!("{}", c.code());
    }

    #[test]
    fn test_eval1() {
        // (\x.x) y
        let id_fn = Expr::abstraction("x", Expr::variable("x"));
        let apply_y = Expr::application(id_fn, Expr::variable("y"));
        apply_y.eval();
    }
}
