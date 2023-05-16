# Rhai

Rhai is a simple scripting language that is embedded in HEMTT for [Hooks](hooks/index.md.md) and [Scripts](scripts/index.md.md).
It has a syntax similar to Javascript, and uses types similar to Rust.

A few examples of Rhai are provided below, but this is not a complete reference.
The full [Lanuage Reference](https://rhai.rs/book/ref/index.html) can be found in the [Rhai Book](https://rhai.rs/book).

## Variables

Variables are defined using `let` and `const`, variables are dynamically typed.
Variables defined with `const` cannot be changed.

```js
let x; // ()
let x = 1; // (i32)
let x = 1.1; // (f32)
const x = "hello"; // (string)
```

### Shadowing

Variables are shadowed in the current scope when redefined.

```js
let x = 1;
print(x); // 1

{
    let x = 2;
    print(x); // 2
}

print(x); // 1
```

## Logic

### If

```js
if foo() {
    print("foo is true");
} else if x == 2 {
    print("x is 2");
} else {
    print("foo is false and x is not 2");
}
```

If statements can be used as expressions.

```js
let x = if foo() {
    1
} else {
    2
};
```
