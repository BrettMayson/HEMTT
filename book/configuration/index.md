# Configuration

Every HEMTT project requires a `.hemtt/project.toml` file. This file contains all the information HEMTT needs to build your mod.

Previous versions of HEMTT supported a `hemtt.json` or `hemtt.toml` file, but these are no longer supported.

## Minimum Configuration

The minimum configuration only requires a `name` and `prefix` to be set.

```toml,fp=.hemtt/project.toml
name = "Advanced Banana Environment"
prefix = "abe"
```

You can read more about these options on the [Minimum Configuration](./minimum.md) page.

## Version

HEMTT uses a custom version format based on standards in the Arma 3 community. You can read more about it on the [Version](./version.md) page.

## Project

You can additionally configure optional settings for your project.

### Main Prefix

The `mainprefix` option allows you to set the root prefix for your project, used before the `prefix` option. This is currently only used by [`hemtt launch`](../commands/launch.md).

```toml,fp=.hemtt/project.toml
mainprefix = "z"
```

It should match the `$PBOPREFIX$` file in each addon folder.

```txt,fp=addons/main/$PBOPREFIX$
z\abe\addons\main
```

### Files

You can add a list of files to be copied to the build directory. This is useful for including files that are not part of addons, such as a `README.md`, `LICENSE`, logos, or extensions. To include a folder, you must use a glob pattern that matches all files in that folder.

```toml,fp=.hemtt/project.toml
[files]
include = [
    "mod.cpp",        # These files are copied to the build directory by default
    "meta.cpp",       # They do not need to be added to your list
    "LICENSE",
    "logo_ca.paa",
    "logo_co.paa",
    "python_code/**", # Copy the folder "python_code" including all its files
]
exclude = [
    "*.psd",          # By default this list is empty
    "addons/main/README.md",
]
```

#### include

By default, those 5 files are included in the build directory if they exist in the root of your project. You do not need to add them to your list. Additional files or [glob paths](<https://en.wikipedia.org/wiki/Glob_(programming)>) can be added to the list.

```toml,fp=.hemtt/project.toml
[files]
include = [
    "mod.cpp",      # These files are copied to the build directory by default
    "meta.cpp",     # They do not need to be added to your list
    "LICENSE",
    "logo_ca.paa",
    "logo_co.paa",
]
```

#### exclude

By default, no files are excluded from PBOs. You can add files or [glob paths](<https://en.wikipedia.org/wiki/Glob_(programming)>) to the list.

```toml,fp=.hemtt/project.toml
[files]
exclude = [
    "*.psd",        # By default this list is empty
    "addons/main/README.md",
]
```

### properties

You can add a list of properties to be added to every PBO.

```toml,fp=.hemtt/project.toml
[properties]
author = "ABE Team"
url = "https://github.com/ABE-Mod/ABE"
```

### Preprocessor

#### runtime_macros

Some runtime macros can be enabled to allow their use in included files. This option should not be enabled unless you need it, and understand what it does.

A table of the currently supported runtime macros is:

| Macro | Value | Description |
|-------|-----------|-------------|
| `__A3_DEBUG__` | `0` | Indicates if the game is running in debug mode. |
| `__A3_DIAG__` | `0` | Indicates if the game is using the _diag build. |
| `__A3_EXPERIMENTAL__` | `0` | Indicates if the game is running an experimental build (dev or profiling branch). |
| `__A3_PROFILING__` | `0` | Indicates if the game is running in profiling mode. (dev branch diag binary or profiling branch profiling binary)  |

```toml,fp=.hemtt/project.toml
[preprocessor]
runtime_macros = true
```

### Signing

#### authority

You can specify the authority to use for BI Signing. By default, this will be `{prefix}_{version}`, e.g. `abe_3.1.0`.

```toml,fp=.hemtt/project.toml
[signing]
authority = "my_authority"
```

#### version

You can specify the version of BI Signing to use. The default is `3`. This should not be changed unless you know what you are doing.

```toml,fp=.hemtt/project.toml
[signing]
version = 3
```
