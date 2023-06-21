# Addon Configuration

In addition to `.hemtt/project.toml`, HEMTT also supports an optional `addon.toml` in each addon folder.

**_/addons/banana/addon.toml_**

```toml
no_bin = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]

preprocess = false # Default: true

exclude = [
    "data/*.psd",
]

[properties]
iso = "14001"
```

## no_bin

The `no_bin` key is an array of glob patterns that will be excluded from binarization and packed as is.

**_/addons/banana/addon.toml_**

```toml
no_bin = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]
```

## preprocess

The `preprocess` key is a boolean that determines if the addon `config.cpp` should be preprocessed. This is not recommended, and should only be used when required, such as when using `__has_include`.

When it is required, it is recommended to create a separate addon to house any optional config, with the minimum amount of code required to make it work. Disabling preprocessing will allow you to ship invalid config, which could cause issues for your players. It will also cause slower load times when the config is valid.

**_/addons/banana/addon.toml_**

```toml
preprocess = false # Default: true
```

## exclude

The `exclude` key is an array of glob patterns that will be excluded from the PBO.

**_/addons/banana/addon.toml_**

```toml
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
