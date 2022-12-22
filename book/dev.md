# hemtt dev

<code>
hemtt dev <a href="#binarize">[-b]</a>
</code>

`hemtt dev` is designed to help your development workflows. It will build your mod into `.hemtt/dev`, with links back to the original addon folders. This allows you to use [file-patching](#file-patching) with optional mods for easy development.

## Configuration

**hemtt.toml**

```toml
[hemtt.dev]
exclude = ["addons/unused"]
```

### exclude

A list of addons to exclude from the development build. Includes from excluded addons can be used, but they will not be built or linked.

## Binarize

By default, `hemtt dev` will not binarize any files, but rather pack them as-is. Binarization is often not needed for development, but can be enabled with the `-b --binarize` flag.
