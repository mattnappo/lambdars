use lambdars_macro::*;

#[test]
fn test_swap() {
    let a = "aaa";
    let b = "bbb";

    let t = lambda! {
        @input(a, b) // capture `a` and `b` from outer scope

        (Lx.Ly. y x) a b   // swap
    };
    println!("{t:?}"); // prints ("bbb", "aaa")
}

#[test]
fn test_copy() {
    let a = "aaa";

    let t = lambda! {
        @input(a)

        (Lx.x x) a
    };
    println!("{t:?}"); // prints ("aaa", "aaa")
}
