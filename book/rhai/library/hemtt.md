# HEMTT

The `HEMTT` constant gives access to information and the ability to modify the build process.

## `version()`

Returns the version of HEMTT.

```js
HEMTT.version().to_string(); // "1.5.0"
HEMTT.version().major(); // 1
HEMTT.version().minor(); // 4
HEMTT.version().patch(); // 0
HEMTT.version().build(); // ""
```

## `project()`

Returns the project information.

See more about the [Project](project.md) library.

```js
HEMTT.project().version().to_string(); // "1.3.0.1052"
```

## `mode()`

Returns the current mode of HEMTT, one of:

- dev
- launch
- build
- release

```js
HEMTT.mode(); // "release"
```

## `is_dev()`

Returns true if the current mode is `dev`.

```js
HEMTT.is_dev(); // false
```

## `is_launch()`

Returns true if the current mode is `launch`.

```js
HEMTT.is_launch(); // false
```

## `is_build()`

Returns true if the current mode is `build`.

```js
HEMTT.is_build(); // false
```

## `is_release()`

Returns true if the current mode is `release`.

```js
HEMTT.is_release(); // true
```
