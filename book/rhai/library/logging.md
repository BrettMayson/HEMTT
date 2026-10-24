# Logging

## Rhai

Rhai has two built in functions for logging, `print` and `debug`.

### `print(string)`

Prints a string to the console.

```js
print("Hello World!");
```

```sh
 INFO [post_release/test.rhai] Hello World!
```

### `debug(any)`

Prints a representation of the value to the console if the `--debug` flag is passed to HEMTT.

```js
debug(HEMTT.version().to_string());
debug(HEMTT.project().version.major());
```

```sh
DEBUG [post_release/test.rhai] "1.22.1"
DEBUG [post_release/test.rhai] 1
```

## HEMTT

HEMTT provides additional logging functions.

### `info(string)`

Prints a string to the console. Same functionality as `print`.

```js
info("Hello World!");
```

```sh
 INFO [post_release/test.rhai] Hello World!
```

### `warn(string)`

Prints a string to the console with a warning prefix.

```js
warn("Hello World!");
```

```sh
 WARN [post_release/test.rhai] Hello World!
```

### `error(string)`

Prints a string to the console with an error prefix.

```js
error("Hello World!");
```

```sh
ERROR [post_release/test.rhai] Hello World!
```

### `fail(string)`

Prints a string to the console with an error prefix, HEMTT will mark the build as failed and exit immediately.

```js
fail("Hello World!");
```

```sh
ERROR [post_release/test.rhai] Hello World!
```

### `fatal(string)`

Similar to `fail(string)`, but indicates a fatal error and HEMTT will print an error showing where the fatal was called.

```js
fatal("Hello World!");
```

```sh
ERROR [post_release/test.rhai] Hello World!
error[BHE4]: Script /.hemtt/hooks/post_release/test.rhai failed at runtime
  ┌─ .hemtt/hooks/post_release/test.rhai:6:5
  │
6 │     fatal("Hello World!");
  │     ^ runtime error: Script called fatal (line 6, position 5)
```
