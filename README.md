# Lambdars
Embed Lambda Calculus in your Rust programs!

Lambdars provides a `lambda!` macro which allows you to write Lambda Calculus programs in Rust.

## Example 1: Hello World

Here is the simplest possible usage of `lambda!`.

```rust
let a = 1;
let out = lambda! {
    @input(a) // capture `a` from the outer scope

    (Lx.x) a  // apply `a` to the identity function
};
assert_eq!(out, a);
```

A `lambda!` macro consists of two parts: an `@input` decorator and a Lambda Calculus expression. The `@input(...)` decorator takes in a list of identifiers from the outer scope. If `a` is denoted as an input with `@input(a)`, then whenever `a` appears in the Lambda Calculus expression, it refers to the variable `a` in the outer scope.

In this case, `(Lx.x) a` reduces to `a`, which is bound to the literal `1`. Since `a` is marked as an `@input`, the enture `lambda!` block returns `a`.

## Example 2: Swap and Copy

Here's how we can "swap" two values.

```rust
let a = 1;
let b = 2;

let t = lambda! {
    @input(a, b) // capture `a` and `b` from outer scope

    (Lx. Ly. y x) a b   // swap
};
assert_eq!(t, (2, 1));
```

In this example, `lambda!` reduces the given expression as much as possible and returns the result as a tuple. For clarity, here are the reductions:
```lisp
((Lx. (Ly. (y x))) a) b
--> (Ly. (y a)) b
--> (b a)
```
Thus, `lambda!` returns the tuple `(b, a)`.

We can also write a `lambda!` that "copies" a value. This is just the expression `Lx. (x x)`.

```rust
let a = 1;
let t = lambda! {
    @input(a)

    (Lx. (x x)) a
};
assert_eq!(t, (1, 1));
```

## Example 3: Complex Output

This example demonstrates the nesting structure of `lambda!` outputs.

```rust
let a = 1;
let b = 2;
let c = "ccc";
let t = lambda! {
    @input(a, b, c)

    (Lx. Ly. y x) a b c
};
assert_eq!(t, ((2, 1), "ccc"));
```

Currently, `lambda!` requires that the reduced expression takes the form
```
S ::= (<S>, <S>)
  ::= <input variable>
```
where an "input variable" is any identifier within the `@input(...)` decorator.

In other words, the given expression within a `lambda!` can reduce only to applications of variables. In the above example, the given expression reduces to `(a b) c`, and `lambda!` returns a tuple `((a, b), c)`.

## Example 4: Logic Gates
In this example we build a `NOT` gate.

If we interpret `Lx. Ly. x` to be `TRUE`, and `Lx. Ly. y` to be `FALSE`, then a `NOT` gate can be written as `Lt. (t FALSE TRUE)`.

```rust
let a = 1;
let b = 2;
let not_true = lambda! {
    @input(a, b)
    (Lt. (t (Lx.Ly.y) (Lx.Ly.x)))   // NOT gate
        (Lx.Ly.x) a b               // apply TRUE to NOT
};
// NOT(TRUE) --> FALSE, and (FALSE a b) --> b
assert_eq!(not_true, b);

let not_false = lambda! {
    @input(a, b)
    (Lt. (t (Lx.Ly.y) (Lx.Ly.x)))   // NOT gate
        (Lx.Ly.y) a b               // apply FALSE to NOT
};
// NOT(FALSE) --> TRUE, and (TRUE a b) --> a
assert_eq!(not_false, a);
```

## Notes

⚠️ This project is not complete ⚠️

### Currying Shorthand Notation
A common shorthand notation for functions with multiple inputs is to only write one lambda. For example, `Lx. Ly. E` is often written as `Lxy. E`. 

Lambdars intentionally does not support this notation, and will instead interpret `Lxy. E` as a function that binds the variable `xy`. Thus, `Lxyz.xyz` is actually just the identity function `Lx.x`.

### Motivation

This project is inspired by [Crepe](https://github.com/ekzhang/crepe).
