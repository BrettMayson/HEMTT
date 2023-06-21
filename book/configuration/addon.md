# Addon Configuration

In addition to `.hemtt/project.toml`, HEMTT also supports an optional `addon.toml` in each addon folder.

**_/addons/banana/addon.toml_**

```toml
[binarize]
enabled = false # Default: true
exclude = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]

[preprocess]
enabled = false # Default: true
exclude = [
    "missions/**/description.ext",
]

[files]
exclude = [
    "data/*.psd",
]

[properties]
iso = "14001"
```

## binarize

HEMTT's binarization of addons can be disabled entirely by setting `binarize.enabled` to `false`, or disabled for specific files by adding glob patterns to `binarize.exclude`.

**_/addons/banana/addon.toml_**

```toml
[binarize]
enabled = false # Default: true
exclude = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]
```

## preprocess

HEMTT's preprocessing of addons can be disabled entirely by setting `preprocess.enabled` to `false`, or disabled for specific files by adding glob patterns to `preprocess.exclude`.

When it is required to disable preprocessing of `config.cpp`, it is recommended to create a separate addon to house any optional config, with the minimum amount of code required to make it work. Disabling preprocessing will allow you to ship invalid config, which could cause issues for your players. It will also cause slower load times when the config is valid.

**_/addons/banana/addon.toml_**

```toml
[preprocess]
enabled = false # Default: true
exclude = [
    "missions/**/description.ext",
]
```

## files

`files.exclude` is an array of glob patterns that will be excluded and not packed into the PBO.

**_/addons/banana/addon.toml_**

```toml
[files]
exclude = [
    "data/*.psd",
]
```

## properties

Much like the `properties` key in `.hemtt/project.toml`, the `properties` key in `addon.toml` allows you to add custom properties to the PBO.

**_/addons/banana/addon.toml_**

```toml
[properties]
iso = "14001"
```
