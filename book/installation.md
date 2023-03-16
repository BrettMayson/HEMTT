# Installation

## Download

The latest HEMTT release can be downloaded from the [GitHub releases page](https://github.com/brettmayson/HEMTT/releases).

Builds are available for Windows and Linux.

## Installation (Project Local)

The HEMTT executable can be placed in the root of your project, and used from there. It is strongly recommended not to add it to your version control system.

HEMTT can then be ran from a terminal in the root of your project with `.\hemtt.exe` on Windows, or `./hemtt` on Linux.

## Installation (Global)

HEMTT can be installed globally on your system, and used from anywhere.

The HEMTT executable can be placed in any directory on your system, and added to your `PATH` environment variable.

HEMTT can then be ran from any terminal with `hemtt`.

## Crates

If you have [Rust](https://www.rust-lang.org/) installed, you can install HEMTT from [crates.io](https://crates.io/crates/hemtt) with `cargo install hemtt`.

## Compile from Source

HEMTT can be compiled from [source](https://github.com/brettmayson/HEMTT) using [Rust](https://www.rust-lang.org/).

HEMTT usually requires the latest stable version of Rust, older versions may work but are not supported.

You can use the `cargo install --path bin` command to install HEMTT while in the root of the repository.
