# Commands

## Setup

- [hemtt new](/commands/new.md) - Create a new project

## Development

- [hemtt check](/commands/check.md) - Check the project for errors
- [hemtt dev](/commands/dev.md) - Build the project for local development
- [hemtt launch](/commands/launch.md) - Launch Arma 3 with your mod and dependencies
- [hemtt build](/commands/build.md) - Build the project for local testing

## Release

- [hemtt release](/commands/release.md) - Build the project for release

## Options

### --just

The [`build`](/commands/build.md) and [`dev`](/commands/dev.md) commands can be used to build a single addon. It can be used multiple times to build multiple addons.

```bash
hemtt build --just myAddon
```

> [!CAUTION]
> It is advised to only use this on very large projects that take a long time to build.
> It is advised to only use this after running the command once without `--just` to ensure all addons are built.
> Anytime you run any git commands that can modify files, you should run without `--just` to ensure all addons are up to date.
> Before reporting any unexpected behavior, try running without `--just` first.

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

> [!NOTE]
> The full log can also be found at `.hemttout/latest.log` after each build
