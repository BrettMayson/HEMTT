# Project

## `version()`

Returns the project version.

```ts
HEMTT.project().version().to_string(); // "1.3.0.1052"
HEMTT.project().version().to_string_short(); // "1.3.0"
HEMTT.project().version().major(); // 1
HEMTT.project().version().minor(); // 3
HEMTT.project().version().patch(); // 0
HEMTT.project().version().build(); // 1052
```

## `name()`

Returns the project name.

```ts
HEMTT.project().name(); // "Advanced Banana Environment"
```

## `prefix()`

Returns the project prefix.

```ts
HEMTT.project().prefix(); // "abe"
```
