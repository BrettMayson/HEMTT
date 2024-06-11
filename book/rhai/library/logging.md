# Logging

## Rhai

Rhai has two built in functions for logging, `print` and `debug`.

### `print(string)`

Prints a string to the console.

```ts
print("Hello World!");
```

```sh
 INFO [post_release/test.rhai] Hello World!
```

### `debug(any)`

Prints a representation of the value to the console if the `--debug` flag is passed to HEMTT.

```ts
debug(HEMTT.version().to_string());
debug(HEMTT.project().version.major());
```

```sh
DEBUG [post_release/test.rhai] "1.12.4"
DEBUG [post_release/test.rhai] 1
```

## HEMTT

HEMTT provides additional logging functions.

### `info(string)`

Prints a string to the console. Same functionality as `print`.

```ts
info("Hello World!");
```

```sh
 INFO [post_release/test.rhai] Hello World!
```

### `warn(string)`

Prints a string to the console with a warning prefix.

```ts
warn("Hello World!");
```

```sh
 WARN [post_release/test.rhai] Hello World!
```

### `error(string)`

Prints a string to the console with an error prefix.

```ts
error("Hello World!");
```

```sh
ERROR [post_release/test.rhai] Hello World!
```

### `fatal(string)`

Prints string to the console with an error prefix, HEMTT will mark the build as failed and exit.

```ts
fatal("Hello World!");
```

```sh
ERROR [post_release/test.rhai] Hello World!
error: Hook signaled failure: post_release/test.rhai
```
