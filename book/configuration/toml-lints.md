# TOML Lints

In addition to HEMTT's built-in lints, you can define your own simple pattern-based lints without writing any code. These are useful for enforcing project-specific conventions that aren't covered by a built-in, configurable lint.

TOML lints are defined as `.toml` files inside a `.hemtt/lints` directory. Every `.toml` file found in that folder is loaded automatically, and each file defines a single lint.

## Defining a Lint

A lint file matches raw file contents against one or more regular expressions. Matches are reported with the `L-TOML` diagnostic code.

```toml,fp=.hemtt/lints/merge-conflict.toml
name = "merge-conflict"
severity = "Error"
message = "Unresolved merge conflict marker"
note = "a git merge conflict was left in this file"
help = "resolve the conflict before committing"
label = "conflict marker"

[sqf.file]
patterns = ["^(<<<<<<<|=======|>>>>>>>)"]
```

### Fields

- `name` - an optional name for the lint, for your own reference.
- `severity` - `"Error"`, `"Warning"`, or `"Help"`. Defaults to `Warning` if omitted.
- `message` - the main message shown for the diagnostic.
- `note` - additional context shown below the diagnostic.
- `help` - a suggestion for how to fix the issue.
- `label` - the message shown under the highlighted span.

### Targets

Patterns are grouped by the kind of file they should run against, and then by a target within that file type. Currently, the only supported target for both is `file`, which matches against the raw source of the whole file.

- `[sqf.file]` - runs against the full contents of `.sqf` files.
- `[config.file]` - runs against the full contents of `.cpp`/`.hpp` config files.

Each target takes a `patterns` array of one or more [Rust regex](https://docs.rs/regex/latest/regex/#syntax) patterns. A lint can define both `sqf.file` and `config.file` sections if it should apply to both file types, and a single lint file can only report one message, note, help, and label for all of its patterns.

```toml,fp=.hemtt/lints/todo.toml
severity = "Help"
message = "TODO comment found"
note = "leftover TODO markers are easy to lose track of"
label = "todo"

[sqf.file]
patterns = ["(?i)//\\s*todo"]

[config.file]
patterns = ["(?i)//\\s*todo"]
```

Every match of every pattern in a file produces a separate diagnostic at the matched location.

> [!NOTE]
> TOML lints are intentionally simple. If you need more advanced analysis, such as understanding SQF syntax or config structure, consider [requesting a built-in lint](https://github.com/BrettMayson/HEMTT/issues) instead.
