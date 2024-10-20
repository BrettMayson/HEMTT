# hemtt check

<pre><code>Check your project

Usage: hemtt check [OPTIONS]

Options:
    <a href="index.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt check` is the quickest way to check your project for errors. All the same checks are run as [`hemtt dev`](./dev), but it will not write files to disk, saving time and resources.
