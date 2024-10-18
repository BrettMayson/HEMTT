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
curl -sSf https://hemtt.dev/install.sh | bash
```

The script can be ran again to update HEMTT.

## Manual Download

The latest HEMTT release can be downloaded from the [GitHub releases page](https://github.com/brettmayson/HEMTT/releases).

Builds are available for Windows, Linux, and MacOS.

## Compile from Source

HEMTT can be compiled from [source](https://github.com/brettmayson/HEMTT) using [Rust](https://www.rust-lang.org/).

HEMTT usually requires the latest stable version of Rust, older versions may work but are not supported.

You can use the `cargo install --path bin` command to install HEMTT while in the root of the repository.
