# Commands

## Development

- [hemtt new](commands-new.md) - Create a new project
- [hemtt dev](commands-dev.md) - Build the project for local development
- [hemtt launch](commands-launch.md) - Launch Arma 3 with your mod and dependencies
- [hemtt build](commands-build.md) - Build the project for local testing

## Release

- [hemtt release](commands-release.md) - Build the project for release

## Global Options

### -t, --threads

Number of threads to use, defaults to the number of CPUs.

```bash
hemtt ... -t 4
```

### -v

Verbosity level, can be specified multiple times.

```bash
hemtt ... -v # Debug
hemtt ... -vv # Trace
```

When running inside a CI platform like GitHub Actions, the output will always be set to trace.

```admonish note
The full log can also be found at `.hemttout/latest.log` after each build
```
