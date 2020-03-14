# Templating
Templating is used to insert data from your project into various config options. [Handlebars](https://github.com/sunng87/handlebars-rust) is used as the templating engine.

```toml
sig_name = "{{version}}-{{date \"%y%m%d\"}}"
```
This would result in something like `ACE_zeus.pbo.3.12.5.40-190227.bisign`

# Variables
## name
The name of the HEMTT project.

## prefix
The prefix of the HEMTT project.

## author
The author of the HEMTT project.

## version
The version of the HEMTT project. See [project#version](/project.md?id=version).

## semver
The version of the HEMTT project as an object.

```cpp
#define MAJOR 1
#define MINOR 5
#define PATCH 3
#define BUILD rc1
```

| major | minor | patch | build |
|-------|-------|-------|-------|
| 1     | 5     | 3     | rc1   |

```
The build is {{semver.build}}
```
Output:
```
The build is rc1
```

# Helpers
## date
Date can be used to get information about the current date and time. HEMTT uses [chrono specifiers](https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html#specifiers) for formatting.

```handlebars
{{date \"%y%m%d\"}}
```

## git
Git helper can be used to get information about the git repository.

Parameters:
- `id <number>`: id (SHA-1) of HEAD revision truncated to `<number>` characters _(from 1 to 40, default: 8)_

```handlebars
{{git \"id 8\"}}
```
