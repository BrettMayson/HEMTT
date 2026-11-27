# Addon Configuration

In addition to `.hemtt/project.toml`, HEMTT also supports an optional `addon.toml` in each addon folder.

```toml,fp=addons/banana/addon.toml
[binarize]
enabled = false # Default: true
exclude = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]

[rapify]
enabled = false # Default: true
exclude = [
    "missions/**/description.ext",
]

[files]
exclude = [
    ".vscode/**/*", # Exclude all files in the .vscode folder
    "data/*.psd",
]

[sqfc]
exclude = [
    "functions/fnc_versionSpecific.sqf",
]
exclude_temporary = [
    "functions/fnc_largeData.sqf",
]

[properties]
iso = "14001"

# ignore mismatched $PBOPREFIX$ for addon
ignore_pboprefix = true # Default: false
```

## binarize

HEMTT's binarization of addons can be disabled for the addon by setting `binarize.enabled` to `false`, or disabled for specific files by adding glob patterns to `binarize.exclude`.

```toml,fp=addons/banana/addon.toml
[binarize]
enabled = false # Default: true
exclude = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]
```

## rapify

HEMTT's preprocessing & rapifying of addon configs can be disabled for the addon by setting `rapify.enabled` to `false`, or disabled for specific files by adding glob patterns to `rapify.exclude`.

When it is required to disable preprocessing & rapifying of `config.cpp`, it is recommended to create a separate addon to house any optional config, with the minimum amount of code required to make it work. Disabling preprocessing & rapifying will allow you to ship invalid config, which could cause issues for your players. It will also cause slower load times when the config is valid.

```toml,fp=addons/banana/addon.toml
[rapify]
enabled = false # Default: true
exclude = [
    "missions/**/description.ext",
]
```

## sqfc

HEMTT compiles SQF to SQFC (compiled SQF) for faster loading. Files listed here are still linted, but their SQFC is not included in the PBO, so the game uses the plain SQF.

Files that check `__A3_DEBUG__` (e.g. `#ifdef __A3_DEBUG__`) are compiled twice, and the SQFC includes a second code block that the game uses when started with `-debug`. No configuration is needed.

`sqfc.exclude` is an array of glob patterns for files that should never be compiled to SQFC.

Files that check built-in macros HEMTT does not support, such as `#if __GAME_VER_MIN__`, should instead use `#pragma hemtt flag pe26_ignore_unsupported_builtin` in the file. HEMTT then skips SQFC for that file automatically, so the game evaluates the condition. See [Preprocessor Flags](../lints/preprocessor.md#preprocessor-flags).

`sqfc.exclude_temporary` is an array of glob patterns for files that HEMTT can't compile yet, for example a file with more unique constants than SQFC supports. HEMTT still tries to compile them, and warns once they compile successfully so the entry can be removed.

If a file can't be compiled and is not in either list, the build fails.

```toml,fp=addons/banana/addon.toml
[sqfc]
exclude = [
    "functions/fnc_versionSpecific.sqf",
]
exclude_temporary = [
    "functions/fnc_largeData.sqf",
]
```

## files

`files.exclude` is an array of glob patterns that will be excluded and not packed into the PBO.
It is important to note that this matches against files, not folders. To exclude a folder, you must use a glob pattern that matches all files in that folder.

```toml,fp=addons/banana/addon.toml
[files]
exclude = [
    ".vscode/**/*", # Exclude all files in the .vscode folder
    "data/*.psd",
]
```

## properties

Much like the `properties` key in `.hemtt/project.toml`, the `properties` key in `addon.toml` allows you to add custom properties to the PBO.

```toml,fp=addons/banana/addon.toml
[properties]
iso = "14001"
```
