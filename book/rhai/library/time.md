# Time

## `date(string)`

Returns the current date in the given format.

```ts
date("[year]-[month]-[day] [hour]:[minute]:[second]"); // {{ time_1 }}
date("[year repr:last_two][month][day]"); // {{ time_2 }}
```

You can find the full list of format specifiers [here](https://time-rs.github.io/book/api/format-description.html#components).
