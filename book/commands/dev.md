# hemtt dev

<pre><code>Build your mod for local development and testing.

Usage: hemtt dev [OPTIONS]

Options:
    <a href="#-b---binarize">-b, --binarize</a>
        Use BI's binarize on supported files
        
    <a href="build.md#--no-rap">--no-rap</a>
        Do not rapify files

    <a href="#-o---optional">-o, --optional &lt;optional&gt;</a>
        Include an optional addon folder

    <a href="#-o---all-optionals">-O, --all-optionals</a>
        Include all optional addon folders

    <a href="index.md#--just">--just &lt;just&gt;</a>
        Only build the specified addon

    <a href="index.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt dev` is designed to help your development workflows. It will build your mod into `.hemttout/dev`, with links back to the original addon folders. This allows you to use [file-patching](#file-patching) with optional mods for easy development.

## Configuration

**.hemtt/project.toml**

```toml
[hemtt.dev]
exclude = ["addons/unused"]
```

### exclude

A list of addons to exclude from the development build. Includes from excluded addons can be used, but they will not be built or linked.

## Options

### -b, --binarize

By default, `hemtt dev` will not binarize any files, but rather pack them as-is. Binarization is often not needed for development, but can be enabled with the `-b --binarize` flag.

```bash
hemtt dev -b
```

### -o, --optional <optional>

Include an optional addon folder. This can be used multiple times to include multiple optional addons.

```bash
hemtt dev -o caramel -o split
```

### -O, --all-optionals

Include all optional addon folders.

```bash
hemtt dev -O
```
