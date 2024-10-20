# hemtt utils config inspect

<pre><code>Inspect a Config

Usage: hemtt utils config inspect [OPTIONS] &lt;config&gt;

Arguments:
  &lt;config&gt;
        Config to inspect

Options:
  -v...
        Verbosity level

  -h, --help
        Print help (see a summary with '-h')
</code>
</pre>

Provides information about a Config.

This is the same as `hemtt utils inspect` but will assume the file is a Config.

In some cases the output might be cut off in the terminal. Adjust the `terminal.integrated.scrollback` setting in VS Code if necessary.
