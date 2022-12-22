# hemtt build

<code>
hemtt build
</code>

`hemtt build` will build your mod into `.hemtt/build`. It will binarize all applicable files, and will not create folder links like `hemtt dev`.

It is intended to be used for testing your mod locally before release.

## Configuration

**hemtt.toml**

```toml
[hemtt.build]
optional_mod_folders = false # Default: true
```

### optional_mod_folders

By default, `hemtt build` will create separate mods for each optional mod folder.
