# Commands

## Development

- [hemtt new](./new.md) - Create a new project
- [hemtt dev](./dev.md) - Build the project for local development
- [hemtt launch](./launch.md) - Launch Arma 3 with your mod and dependencies
- [hemtt build](./build.md) - Build the project for local testing

## Release

- [hemtt release](./release.md) - Build the project for release

## Options

### --just

The [`build`](./build.md) and [`dev`](./dev.md) commands can be used to build a single addons. It can be used multiple times to build multiple addons.

```bash
hemtt build --just myAddon
```

```admonish danger
It is advised to only use this on very large projects that take a long time to build.
It is advised to only use this after running the command once without `--just` to ensure all addons are built.
Anytime you run any git commands that can modify files, you should run without `--just` to ensure all addons are up to date.
Before reporting any unexpected behavior, try running without `--just` first.
```

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
