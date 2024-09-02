use lambdars_macro::*;

#[test]
fn test1() {
    lambda! {
        (Lx.x x) (Ly. y x) a
    };

    //println!("x = {}", x);
}
