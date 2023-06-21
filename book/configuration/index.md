# Configuration

Every HEMTT project requires a `.hemtt/project.toml` file. This file contains all the information HEMTT needs to build your mod.

Previous versions of HEMTT supported a `hemtt.json` or `hemtt.toml` file, but these are no longer supported.

## Minimum Configuration

The minimum configuration only requires a `name` and `prefix` to be set.

**.hemtt/project.toml**

```toml
name = "Advanced Banana Environment"
prefix = "abe"
```

You can read more about these options on the [Minimum Configuration](./minimum.md) page.

## Version

HEMTT uses a custom version format based on standards in the Arma 3 community. You can read more about it on the [Version](./version.md) page.

## Project

You can additionally configure optional settings for your project.

### Main Prefix

The `mainprefix` option allows you to set a the root prefix for your project, used before the `prefix` option. This is currently only used by [`hemtt launch`](../commands/launch.md).

**.hemtt/project.toml**

```toml
mainprefix = "z"
```

It should match the `$PBOPREFIX$` file in each addon folder.

**addons/main/$PBOPREFIX$**

```txt
z\abe\addons\main
```

### Files

You can add a list of files to be copied to the build directory. This is useful for including files that are not part of addons, such as a `README.md`, `LICENSE`, logos, or extensions.

**.hemtt/project.toml**

```toml
[files]
include = [
    "mod.cpp",      # These files are copied to the build directory by default
    "meta.cpp",     # They do not need to be added to your list
    "LICENSE",
    "logo_ca.paa",
    "logo_co.paa",
]
exclude = [
    "*.psd",        # By default this list is empty
    "addons/main/README.md",
]
```

#### include

By default, those 5 files are included in the build directory if they exist in the root of your project. You do not need to add them to your list. Additional files or [glob paths](<https://en.wikipedia.org/wiki/Glob_(programming)>) can be added to the list.

**.hemtt/project.toml**

```toml
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

**.hemtt/project.toml**

```toml
[files]
exclude = [
    "*.psd",        # By default this list is empty
    "addons/main/README.md",
]
```

### properties

You can add a list of properties to be added to every PBO.

**.hemtt/project.toml**

```toml
[properties]
author = "ABE Team"
url = "https://github.com/ABE-Mod/ABE"
```
