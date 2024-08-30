use std::collections::HashMap;
use tracing::instrument;

type Scope = HashMap<Var, u32>;

/// A variable used in a lambda expression.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Var {
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

    /// Naively substitute all occurrences of `var` with `e` in `self`.
    //#[instrument(skip(self))]
    #[instrument]
    pub fn sub(&self, var: &Var, e: Expr) -> Expr {
        tracing::info!("");
        use Expr::*;
        match self {
            ev @ Variable(v) => {
                // Only sub if there is a ident. Otherwise, its a free var.
                // dbg!(&var);
                // dbg!(&v);
                if var.name == v.name {
                    e.clone()
                } else {
                    ev.clone()
                }
                /*
                match (var.ident, v.ident) {

                    // TODO: the issue is here (perhaps?)
                    (Some(i), Some(j)) if i == j && var.name == v.name => e.clone(),
                    //(None, Some(i)) => {} //(Some(i), None) => {},
                    //(None, None) => {},
                    _ => ev.clone(),
                }
                /*
                if *v == var.name {
                    e.clone()
                } else {
                    ev.clone()
                }
                */
                    */
            }
            Abstraction(v, f) => {
                // TODO: handle case where svar == var?
                if v == var {
                    panic!("uh oh. bad canonicalization?");
                }
                Abstraction(v.clone(), Box::new(f.sub(var, e.clone())))
            }
            Application(e1, e2) => Application(
                Box::new(e1.sub(var, e.clone())),
                Box::new(e2.sub(var, e.clone())),
            ),
        }
    }

    pub fn canonicalize_inner(&self, scope: &Scope, d: u32) -> Expr {
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
    //#[instrument(skip(self))]
    pub fn canonicalize(&self) -> Expr {
        //tracing::info!("");
        self.canonicalize_inner(&HashMap::new(), 0)
    }

    /// Evaluate an expression by performing beta-reduction and alpha-renaming
    /// when necessary.
    #[instrument]
    fn eval_inner(&self) -> Expr {
        tracing::info!("");
        use Expr::*;

        match self.clone() {
            Application(e1, e2) => {
                let (e1, e2) = (Box::new(e1.eval_inner()), Box::new(e2.eval_inner()));
                match *e1.clone() {
                    Abstraction(var, e) => {
                        let s = e.sub(&var, *e2.clone()).eval_inner();

                        /*
                        println!(
                            "β: {} --> [{} \\ {}] --> {}",
                            e.code(),
                            var.code(),
                            e2.code(),
                            s.code()
                        );
                        */
                        s
                    }
                    //e => Application(Box::new(e1.eval_inner()), Box::new(e2.eval_inner())),
                    e => Application(e1, e2),
                }
            }
            other => other,
        }
    }

    fn eval(&self) -> Expr {
        let c = self.canonicalize();
        println!("canonicalized: {}", c.code());
        c.eval_inner()
    }
}

/// Helper for testing.
pub fn eval(e: &Expr) -> Expr {
    let reduced = e.eval();
    println!("{} --> {}", e.code(), reduced.code());
    println!("reduced: {:#?}", reduced);
    reduced
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn tracing() {
        use tracing_subscriber::{prelude::*, Layer};
        let layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(std::io::stdout)
            //.and_then(tracing_subscriber::filter::EnvFilter::from_default_env())
            .boxed();

        tracing_subscriber::registry().with(layer).init();
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
    //#[tracing_test::traced_test]
    fn test_eval2() {
        //tracing();
        tracing_subscriber::fmt::init();
        //let _ = env_logger::builder().is_test(true).try_init();

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

        // [ \x. x (\x. x) (\x. x x) ] a
        // -> a (\x.x) (\x. x x)
        // -> a (\x. x x)
        let apply = Expr::application(outer_abstraction, Expr::variable("a"));
        println!("final");
        let f = eval(&apply);

        // try to reudce again manually from the outside
        let second = eval(&f);
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
    fn test_complex() {
        // ((\f.(\x.f (\y.(\z.y (z z)) (\w.x (w w))))) (\p.(\q.p (q q)))) (\a.\b.a)

        // Define the innermost abstraction
        let inner_abstraction1 = Expr::abstraction("w", Expr::variable("w"));

        // Define the middle-level abstraction
        let middle_abstraction = Expr::abstraction("z", inner_abstraction1.clone());

        // Define the application of the middle-level abstraction
        let inner_application = Expr::application(
            Expr::variable("f"),
            Expr::application(Expr::variable("y"), Expr::variable("y")),
        );

        // Define the inner abstraction
        let inner_abstraction = Expr::abstraction("x", inner_application);

        // Define the application of the inner abstraction
        let application1 = Expr::application(inner_abstraction, middle_abstraction.clone());

        // Define the body of the outer abstraction
        let body = Expr::application(application1, Expr::variable("f"));

        // Define the outer abstraction
        let outer_abstraction = Expr::abstraction("f", body);

        // Define the final application
        let final_application = Expr::application(outer_abstraction, middle_abstraction);
        eval(&final_application);

        // let output = Expr::application(final_application, Expr::variable("a"));
        //eval(&output);
    }
}
