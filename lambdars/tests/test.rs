use lambdars::*;

#[test]
fn test_simple() {
    let a = "aaa";

    let t = lambda! {
        @input(a)
        a
    };
    assert_eq!(t, "aaa");
}
#[test]
fn test_swap() {
    let a = "aaa";
    let b = "bbb";

    let t = lambda! {
        @input(a, b) // capture `a` and `b` from outer scope

        (Lx.Ly. y x) a b   // swap
    };
    assert_eq!(t, ("bbb", "aaa"));
}

#[test]
fn test_copy() {
    let a = "aaa";

    let t = lambda! {
        @input(a)
        (Lx.x x) a
    };
    assert_eq!(t, ("aaa", "aaa"));
}

#[test]
fn test_nesting() {
    let a = 1;
    let b = 2;
    let c = "ccc";
    let t = lambda! {
        @input(a, b, c)
        (Lx.Ly. y x) a b c
    };
    assert_eq!(t, ((2, 1), "ccc"));
}

#[test]
fn test_complex() {
    let t = 2;
    let out = lambda! {
        @input(t)

        (Lx.(Ly.x y)(Lz.z))(La.a a) t
    };
    assert_eq!(out, t);
}

#[test]
fn test_not() {
    let a = 1;
    let b = 2;
    let not_true = lambda! {
        @input(a, b)
        (Lt. (t (Lx.Ly.y) (Lx.Ly.x)))   // NOT gate
            (Lx.Ly.x) a b               // call the NOT gate with TRUE
    };
    // NOT(TRUE) --> FALSE, and (FALSE a b) --> b
    assert_eq!(not_true, b);

    let not_false = lambda! {
        @input(a, b)
        (Lt. (t (Lx.Ly.y) (Lx.Ly.x)))   // NOT gate
            (Lx.Ly.y) a b               // call the NOT gate with FALSE
    };
    // NOT(FALSE) --> TRUE, and (TRUE a b) --> a
    assert_eq!(not_false, a);
}
