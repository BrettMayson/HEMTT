# hemtt utils pbo unpack

<pre><code>Unpack a PBO

Usage: hemtt utils pbo unpack [OPTIONS] &lt;pbo&gt; &lt;output&gt;

Arguments:
  &lt;pbo&gt;     PBO file to unpack
  &lt;output&gt;  Directory to unpack to

Options:
  -v...                    Verbosity level
  -h, --help               Print help
</code></pre>

Unpacks a PBO to a directory.

A `$PBOPREFIX$` file will be created in the output directory containing the prefix of the PBO.

All other properties from the PBO will be saved into `properties.txt`
