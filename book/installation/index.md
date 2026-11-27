# Installation

## Windows

HEMTT can be installed using [Winget](https://github.com/microsoft/winget-cli).

```powershell
winget install hemtt
```

To update HEMTT with winget use:

```powershell
winget upgrade hemtt
```

## Linux & MacOS

HEMTT can be installed using an installer script.

```bash
curl -sSf https://hemtt.dev/install.sh | sh
```

The script installs the HEMTT launcher as `hemtt` in `~/.local/bin`. It requires `curl` and `unzip`, and on macOS it also requires [Homebrew](https://brew.sh).

On first run the launcher downloads the latest HEMTT release. It checks for new releases every 12 hours and updates HEMTT automatically. Running the script again installs the latest launcher.

## Manual Download

The latest HEMTT release can be downloaded from the [GitHub releases page](https://github.com/brettmayson/HEMTT/releases).

Builds are available for Windows, Linux, and MacOS.

## Compile from Source

HEMTT can be compiled from [source](https://github.com/brettmayson/HEMTT) using [Rust](https://www.rust-lang.org/).

HEMTT usually requires the latest stable version of Rust, older versions may work but are not supported.

You can use the `cargo install --path bin` command to install HEMTT while in the root of the repository.

## Arma 3 Location

HEMTT finds Arma 3 through Steam. It is used by [`hemtt dev`](../commands/dev.md) to set up file patching, by [`hemtt launch`](../commands/launch.md) and by [`hemtt doctor`](../commands/doctor.md).

If Arma 3 is installed somewhere Steam doesn't know about, set the `HEMTT_ARMA3_PATH` environment variable to the folder that contains `arma3_x64.exe`. HEMTT always uses this path if it is set.
