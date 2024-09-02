# Lint Configuration

HEMTT runs lints against your config and SQF files to check for errors, common mistakes, and improvements. Some lints can be disabled, configured, or changed in severity.

Lints can be kept in the `project.toml` file under the `lints` section, or in a separate `.hemttout/lints.toml` file. When kept in `lints.toml`, the `lints.` prefix is not required.

See the Analysis section for [Config](../analysis/config.md) and [SQF](../analysis/sqf.md) lints.

## Configuration

```admonish note
Some lints are not able to be disabled or reduced in severity. These are usually critical lints that are required for your project to work correctly.
```

Lint configuration is done in the `hemtt.toml` file in the root of your project.

There are 3 ways to configure a lint:

### Disabling

A lint can be completely disabled by setting it to `false`. Only lints that are not critical can be disabled.

```toml
[lints.sqf]
command_case = false
```

### Severity

A lint can have its severity changed. The severity can be `Error`, `Warning`, or `Help`. Only lints that are not critical errors can have their severity changed.

```toml
[lints.sqf]
command_case = "Warning"
```

### Options

Some lints provide configuration options. Check the documentation for the lint to see what options are available.

```toml
[lints.sqf.command_case]
severity = "Error"
options.ignore = [
    "AGLtoASL",
    "ASLtoAGL",
]
```
