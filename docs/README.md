# HEMTT
Build System for Arma 3 powered by [armake2](https://github.com/KoffeinFlummi/armake2) for Linux and Windows.

[Example Project](https://github.com/synixebrett/HEMTT-Example)

## Using HEMTT
HEMTT is a CLI tool that must be called from the root of your project. HEMTT either needs to be placed in the root and called with `./hemtt`.

## Download
HEMTT is available to download for Linux and Windows. [GitHub Releases](https://github.com/synixebrett/HEMTT/releases)  
Most Windows users will want to use `x86_64-pc-windows-msvc`  
Most Linux users will want to use `x86_64-unknown-linux-gnu`

## Building
Building HEMTT is done with `cargo`, the package manager for Rust. You will need to install the OpenSSL development libraries for your operating system.
HEMTT can be built with `--release` to create a faster, optimized version.
```
cargo build
```
```
cargo build --release
```
### Windows
You will need to install the OpenSSL development libraries. On Windows you can download [precompiled OpenSSL binaries](http://slproweb.com/products/Win32OpenSSL.html) (non-light, 64bit).
