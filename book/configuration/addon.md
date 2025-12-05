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

[properties]
iso = "14001"
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
