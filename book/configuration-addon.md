# Addon Configuration

In addition to `.hemtt/project.toml`, HEMTT also supports an optional `addon.toml` in each addon folder.

***/addons/banana/addon.toml***

```toml
no_bin = [
    "data/*.p3d",
    "data/anim/chop.rtm",
]

preprocess = false # Default: true
```

## No Bin

The `no_bin` key is an array of glob patterns that will be excluded from binarization and packed as is.

## Preprocess

The `preprocess` key is a boolean that determines if the addon `config.cpp` should be preprocessed. This is not recommended, and should only be used when required, such as when using `__has_include`.

When it is required, it is recommended to create a separate addon to house any optional config, with the minimum amount of code required to make it work. Disabling preprocessing will allow you to ship invalid config, which could cause issues for your players. It will also cause slower load times when the config is valid.
