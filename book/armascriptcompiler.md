# ArmaScriptCompiler

```admonish danger
Embeded ArmaScriptCompiler is currently disabled by default due to a privacy issue in the ArmaScriptCompiler. You will need to enable it manually in your `hemtt.toml` file if you wish to use it. Not that output will include the full path of HEMTT's temporary directory, including your username.

Alternatively you can run ASC before HEMTT as part of your CI workflow.
```

HEMTT includes a copy of the [ArmaScriptCompiler](https://github.com/dedmen/ArmaScriptCompiler). It will produce an `.sqfc` file with [SQF Bytecode](https://community.bistudio.com/wiki/SQF_Bytecode) for each `.sqf` file in your project.

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
