# HEMTT

The `HEMTT` constant gives access to information and the ability to modify the build process.

## `version()`

Returns the version of HEMTT.

```js
HEMTT.version().to_string(); // "1.4.0"
HEMTT.version().major(); // 1
HEMTT.version().minor(); // 4
HEMTT.version().patch(); // 0
HEMTT.version().build(); // ""
```

## `project()`

Returns the project information.

See more about the [Project](project.md) library.

```js
HEMTT.project().version().to_string(); // "1.3.0-alpha"
```
