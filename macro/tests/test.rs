use lambdars_macro::*;

#[test]
fn test1() {
    let a = "asd";
    let thing = lambda! {
        //@input(a); // captures a from outer scope
        (Lx.x) a
    };

    println!("thing = {a}");
}

/*
#[test]
fn test1() {
    let func = lambda! {
        @input(a);
        (Lx.x x) (Ly. y x) a
    };

    func(a);
}
*/
