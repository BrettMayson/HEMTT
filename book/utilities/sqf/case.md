# hemtt utils sqf case

<pre><code>Fix capitalization in SQF commands

Usage: hemtt utils sqf [OPTIONS] &lt;path&gt;

Arguments:
  &lt;path&gt;
          Path to the SQF file or a folder to recursively fix

Options:
    -t, --threads
        Number of threads, defaults to # of CPUs

    <a href="../../commands/index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

This will recursively correct all capitalization mistakes in SQF commands.

For example:

```sqf
private _positionASL = GetPosasl Player;
// becomes
private _positionASL = getPosASL player;
```