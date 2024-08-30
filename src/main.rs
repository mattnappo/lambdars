use lambdars::ast::*;

#[tracing::instrument]
fn test_fn() {
    //tracing::info!("helo world");
    //tracing::info!("");
}

fn main() {
    //tracing_subscriber::fmt()
    //   .with_max_level(tracing::Level::TRACE)
    //  .init();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(true)
        .init();

    test_fn();

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
    // -> a (\x.x) (\x. x x)     // WE ARE STUCK HERE!
    // -> a (\x. x x)
    let apply = Expr::application(outer_abstraction, Expr::variable("a"));
    println!("final");
    let f = eval(&apply);

    // try to reudce again manually from the outside
    //let second = eval(&f);
    tracing::debug!("hello world!");
}
