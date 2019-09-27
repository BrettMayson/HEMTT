# HEMTT Project File

The `hemtt.toml` file is used to configure your HEMTT Project. `hemtt.json` is currently supported, but deprecated. `hemtt.toml` will be used if both files are present.

`TOML`
```toml
name = "Advanced Banana Environment"
prefix = "ABE3"
author = "ACE Mod Team"
version = "1.0.0.0"
```

`JSON` (Deprecated)
```json
{
  "name": "Advanced Banana Environment",
  "prefix": "ABE3",
  "author": "ACE Mod Team",
  "version": "1.0.0.0"
}
```

!> JSON is deprecated

# Required Fields

## name
**Type**: String

Long name of your project.
```toml
name = "Advanced Banana Environment"
```
<hr/>

## prefix
**Type**: String

Prefix used for CBA macros and the release directory name.

```toml
prefix = "ABE3"
```

**Example**

```toml
prefix = "ABE3"
version = "1.0.0.0"
```

`hemtt build --release` would create a release in the directory `releases/1.0.0.0/@ABE3/`.
<hr/>

## author
**Type**: String

Author of the project.

```toml
author = "ACE Mod Team"
```
<hr/>

# Optional Fields

## version
**Type**: String

HEMTT will look for `addons/main/script_version.hpp` and use it for the version number. If you are not using the CBA project structure or do not have that file you can add a version number in the HEMTT project file.

```toml
version = "1.0.0.0"
```

If you are using `addons/main/script_version.hpp` the file must be formatted as:
```
#define MAJOR 1
#define MINOR 0
#define PATCH 0
#define BUILD 0
```
- `PATCH` can be substituted with `PATCHLVL`.
<hr/>

## files
**Type**: Array \[String\]

HEMTT will copy the files to the release directory after a successful release build. Supports [glob](http://man7.org/linux/man-pages/man7/glob.7.html) patterns.

```toml
files = [
    "mod.cpp",
    "logo.paa",
    "*.dll"
]
```
<hr/>

## include
**Type**: Array \[String (Path)\]

HEMTT will include matching relative or absolute paths when building.

```toml
include = [
    "./include"
]
```

`./include` will be automatically added on project creation if "include" folder is present.

## exclude
**Type**: Array \[String\]

HEMTT will exclude matching files when building.

```toml
"exclude": [
    "*.psd",
    "*.png",
    "*.tga"
]
```

## optionals
**Type**: Array \[String\]

HEMTT will build the specified addons from the `./optionals` folder.

```json
"optionals": [
    "tracers",
    "particles"
]
```

## folder_optionals
**Type**: Bool

HEMTT will by default build optionals into their own mod folders, which can be directly launched by the user. This can be turned off to build optional PBOs directly into `optionals` folder.

```toml
folder_optionals =  false
```

## skip
**Type**: Array \[String\]

HEMTT will skip building the specified addons.

```toml
skip = [
    "hearing",
    "zeus"
]
```

## headerexts
**Type**: Array \[String\]

HEMTT will apply specified header extensions to each PBO. Supports [templating](/templating.md).

```toml
[headerexts]
    author = "me"
```

## modname
**Type**: String

HEMTT will use the specified mod name (without `@`) to form `@mod` folder. Supports [templating](/templating.md).

```toml
modname = "my_mod"
```

## key_name
**Type**: String

HEMTT will use the specified key name for `.bikey` and `.biprivatekey` names. Supports [templating](/templating.md).

The default is set according to the following table:

| `reuse_private_key` value | Default `key_name`         |
| ------------------------- | ------------------------- |
| `false`                   | `{{prefix}}_{{version}}`  |
| `true`                    | `{{prefix}}`              |

```toml
key_name = "my_key"
```

### Example

```toml
project = "TST"
version = "1.0.0.0"
key_name = "my_key_{{version}}"
```

Above will result in key name of `my_key_1.0.0.0.bikey` and private key name of `my_key_1.0.0.0.biprivatekey`.


## sig_name
**Type**: String

HEMTT will use the specified signature name as part of the full signature (`.bisign`) name. Supports [templating](/templating.md).

```toml
sig_name = "my_custom_name"
```

### Example

```toml
project = "TST"
version = "1.0.0.0"
sig_name = "my-{{version}}"
```

Above will result in signature name of `TST_<addon>.pbo.my-1.0.0.0.bisign` (where `<addon>` is the name of the addon folder), located next to the matching addon PBO.

## sigversion
**Type**: Integer

HEMTT will use the specified signature version.  
Currently Supported: V2, V3 (Experiemental).  
Default: 3

### Example

```toml
sigversion = 3
```

## reuse_private_key

**Type**: bool

If set to `true`, HEMTT will use (and re-use) `releases/keys/{key_name}.biprivatekey`. It will be generated if it doesn't exist.

The default behaviour is to generate a new private key each time and discard it immediately.

!> HEMTT strongly recommends that you only re-use the key if you are making a client-side mod where it will not matter if clients are running different versions of the mod.

```toml
reuse_private_key = false
```
