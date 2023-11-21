# hemtt utils verify

<pre><code>Check a .bisign file against a public key and PBO

Usage: hemtt utils verify [OPTIONS] &lt;pbo&gt; &lt;bikey&gt;

Arguments:
  &lt;pbo&gt;
          PBO to verify

  &lt;bikey&gt;
          BIKey to verify against

Options:

    <a href="../../commands/index.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

This will verify a signed PBO against a public key. This is useful for verifying that a PBO is signed correctly.

It will check:

- The authority matches
- The PBO is correctly sorted
- The hashes match
- A prefix property is present
