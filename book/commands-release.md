# hemtt release

<pre><code>Build a release version your project

Usage: hemtt.exe release [OPTIONS]

Options:
    <a href="commands.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="commands.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt release` will build your mod into `.hemttout/release`. It will create `bisign` files for all addons, and a `bikey` for validation.

It is intended to be used for releasing your mod.

It will create two zip archives in the `releases` folder:
    - `{name}-latest.zip`
    - `{name}-{version}.zip`

## Configuration

`hemtt release` is built the same way as [`hemtt build`](commands-build.md), and will use its configuration.
