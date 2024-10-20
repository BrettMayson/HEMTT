# hemtt localization sort

<pre><code>Sorts the stringtable.xml files in the project.

Usage: hemtt localization sort [OPTIONS]

Options:
    <a href="index.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

## Description

HEMTT will:

1. Sort the Packages in alphabetical order.
2. Sort the Containers in alphabetical order (if any).
3. Sort the Keys in alphabetical order.
4. Sort the Localized Strings in the order of [this table](https://community.bistudio.com/wiki/Stringtable.xml#Supported_Languages)
