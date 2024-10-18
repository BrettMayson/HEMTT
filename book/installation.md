# Installation

## Download

The latest HEMTT release can be downloaded from the [GitHub releases page](https://github.com/brettmayson/HEMTT/releases).

Builds are available for Windows and Linux.

## Recommended Installation (Winget, Global)

### Windows

HEMTT can be installed using [Winget](https://github.com/microsoft/winget-cli).

```powershell
winget install hemtt
```

To update HEMTT with winget use:

```powershell
winget upgrade hemtt
```

### Linux & MacOS

HEMTT can be installed using an installer script.

```bash
curl -sSf https://hemtt.dev/install.sh | bash
```

The script can be ran again to update HEMTT.

## Manual Installation (Global)

HEMTT can be installed globally on your system, and used from anywhere.

The HEMTT executable can be placed in any directory on your system, and added to your `PATH` environment variable.

HEMTT can then be ran from any terminal with `hemtt`.

## Manual Installation (Project Local)

The HEMTT executable can be placed in the root of your project, and used from there.

```admonish warning
It is strongly recommended not to add it to your version control system.
```

HEMTT can then be ran from a terminal in the root of your project with `.\hemtt.exe` on Windows, or `./hemtt` on Linux.

```admonish note
Whenever possible use the global winget installation. That way HEMTT stays up-to-date.
```

## Compile from Source

HEMTT can be compiled from [source](https://github.com/brettmayson/HEMTT) using [Rust](https://www.rust-lang.org/).

HEMTT usually requires the latest stable version of Rust, older versions may work but are not supported.

You can use the `cargo install --path bin` command to install HEMTT while in the root of the repository.
