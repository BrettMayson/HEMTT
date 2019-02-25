# HEMTT.json Project File

The `hemtt.json` file is used to configure your HEMTT Project.

```json
{
  "name": "Advanced Banana Environment",
  "prefix": "ABE3",
  "author": "ACE Mod Team",
  "version": "1.0.0.0"
}
```

# Required Fields

## name
**Type**: String

Long name of your project.
```json
"name": "Advanced Banana Environment"
```
<hr/>

## prefix
**Type**: String

Prefix used for CBA macros and the release directory name.

```json
"prefix": "ABE3"
```

**Example**

```json
"prefix": "ABE3",
"version": "1.0.0.0"
```

`hemtt build --release` would create a release in the directory `releases/1.0.0.0/@ABE3/`.
<hr/>

## author
**Type**: String

Author of the project.

```json
"author": "ACE Mod Team"
```
<hr/>

# Optional Fields

## version
**Type**: String

HEMTT will look for `addons/main/script_version.hpp` and use it for the version number. If you are not using the CBA project structure or do not have that file you can add a version number here.

```json
"version": "1.0.0.0"
```

If you are using `addons/main/script_version.hpp` the file must be formatted as:
```
#define MAJOR 1
#define MINOR 0
#define PATCH 0
#define BUILD 0
```
<hr/>

## files
**Type**: Array \[String\]

HEMTT will copy the files to the release directory after a successful release build.

```json
"files": [
    "mod.cpp",
    "logo.paa"
]
```
<hr/>

## include
**Type**: Array \[String (Path)\]

HEMTT will include matching relative or absolute paths when building.

```json
"include": [
    "./include"
]
```

`./include` will be automatically added on project creation if "include" folder is present.

## exclude
**Type**: Array \[String\]

HEMTT will exclude matching files when building.

```json
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

## skip
**Type**: Array \[String\]

HEMTT will skip building the specified addons.

```json
"skip": [
    "hearing",
    "zeus"
]
```

## headerexts
**Type**: Array \[String\]

HEMTT will apply specified header extensions to each PBO.

```json
"headerexts": [
    "author=me"
]
```

## modname
**Type**: String

HEMTT will use the specified mod name (without `@`) to form `@mod` folder.

```json
"modname": "my_mod"
```

## keyname
**Type**: String

HEMTT will use the specified key name for `.bikey` and `.biprivatekey` names.

```json
"keyname": "my_key"
```
### Replacements

- `{{version}}`: Mod version in format `MAJOR.MINOR.PATCH.BUILD`

### Example

```json
"project": "TST",
"version": "1.0.0.0",
"keyname": "my_key-{{version}}"
```

Above will result in key name of `my_key-1.0.0.0.bikey` and private key name of `my_key-1.0.0.0.biprivatekey`.

## signame
**Type**: String

HEMTT will use the specified signature name as part of the full signature (`.bisign`) name.

```json
"signame": "my_custom_name"
```

### Replacements

- `{{version}}`: Mod version in format `MAJOR.MINOR.PATCH.BUILD`

### Example

```json
"project": "TST",
"version": "1.0.0.0",
"signame": "my-{{version}}"
```

Above will result in signature name of `TST_<addon>.pbo.my-1.0.0.0.bisign` (where `<addon>` is the name of the addon folder), located next to the matching addon PBO.
