# Linting

## SQF Linting

All `.sqf` files will be linted, currently only for preprocessor errors (macros).

### Configuration

**.hemtt/project.toml**

```toml
[lint.sqf]
enabled = false # Default: true
exclude = [
    "addons/main/XEH_preInit.sqf"
]
```

#### enabled

`enabled` is a boolean value that enables or disables linting SQF. It is enabled by default.

#### exclude

`exclude` is an array of strings that are paths to files that should be excluded from linting.
