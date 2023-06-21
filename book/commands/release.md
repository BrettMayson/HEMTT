# hemtt release

<pre><code>Build a release version your project

Usage: hemtt.exe release [OPTIONS]

Options:
    <a href="#--no-sign">--no-sign</a>
        Do not sign the PBOs

    <a href="#--no-archive">--no-archive</a>
        Do not create a zip archive of the release

    <a href="build.md#--no-bin">--no-bin</a>
        Do not binarize files

    <a href="build.md#--no-rapify">--no-rap</a>
        Do not rapify files

    <a href="index.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt release` will build your mod into `.hemttout/release`. It will create `bisign` files for all addons, and a `bikey` for validation.

It is intended to be used for releasing your mod.

It will create two zip archives in the `releases` folder: - `{name}-latest.zip` - `{name}-{version}.zip`

## Configuration

`hemtt release` is built the same way as [`hemtt build`](build.md), and will use its configuration.

```toml
[hemtt.release]
sign = false # Default: true
archive = false # Default: true
```

### sign

If `sign` is set to `false`, a `bikey` will not be created, and the PBOs will not be signed.

```admonish danger
All public releases of your mods should be signed. This will be a requirement of many communities, and is an important security feature.
```

### archive

If `archive` is set to `false`, a zip archive will not be created. The output will be in `.hemttout/release`.

## Options

### `--no-sign`

Do not sign the PBOs or create a `bikey`.

### `--no-archive`

Do not create a zip archive of the release. The output will be in `.hemttout/release`.
