/// A variable used in a lambda expression.
#[derive(Debug)]
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
#[derive(Debug)]
enum Expr {
    Variable(Var),
    Abstraction(Var, Box<Expr>),
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
        match self {
            Expr::Variable(Var { name: v }) => v.to_string(),
            Expr::Abstraction(Var { name: v }, e) => format!("(\\{}. {})", v, &*e.code()),
            Expr::Application(e1, e2) => format!("({} {})", &*e1.code(), &*e2.code()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        // \x.x
        let id_fn = Expr::abstraction("x", Expr::variable("x"));

        // (\x.x) y
        let apply_y = Expr::application(id_fn, Expr::variable("y"));

        dbg!(&apply_y);

        println!("{}", apply_y.code());
    }
}
