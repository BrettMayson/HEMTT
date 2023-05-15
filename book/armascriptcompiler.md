# ArmaScriptCompiler

HEMTT includes a copy of the [ArmaScriptCompiler](https://github.com/dedmen/ArmaScriptCompiler). It will produce an `.sqfc` file with [SQF Bytecode](https://community.bistudio.com/wiki/SQF_Bytecode) for each `.sqf` file in your project.

```admonish info
ArmaScriptCompiler is not available on MacOS.
```

## Configuration

ArmaScriptCompiler requires no configuration and works out of the box.

```toml
[asc]
enabled = false # Default: true
exclude = [
    "/example.sqf",
    "settings/gui.sqf",
]
```

### enabled

`enabled` is a boolean value that enables or disables the ArmaScriptCompiler. It is enabled by default.

### exclude

`exclude` is an array of strings that are paths to files that should be excluded from the ArmaScriptCompiler.
