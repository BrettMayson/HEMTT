# HEMTT

Build System for Arma 3 powered by [armake2](https://github.com/KoffeinFlummi/armake2) for Linux and Windows - Heavy Expanded Mobility Tactical Truck for Arma 3 mods. HEMTT focuses on CBA and ACE3 standards while providing project configurability and additional utilities.

Read this documentation to learn more about it and reference the example project [HEMTT-Example](https://github.com/synixebrett/HEMTT-Example) to see it in action.


## Using HEMTT

HEMTT is a CLI tool that must be called from the root of your project. HEMTT needs to be placed in the project root and called with `./hemtt` on Linux or `hemtt.exe` on Windows. Global install is currently not possible.

Read pages available from the navigation sidebar to learn more about creating mods and building them using HEMTT.


## Download

HEMTT is available for Linux and Windows via [GitHub Releases](https://github.com/synixebrett/HEMTT/releases/latest).
- Most Windows users will want to use `x86_64-pc-windows-msvc`
- Most Linux users will want to use `x86_64-unknown-linux-gnu`


## Building HEMTT

Building HEMTT is done with `cargo`, the package manager for Rust. You will need to install the OpenSSL development libraries for your operating system.

HEMTT can be built with `--release` to create a faster, optimized version.
```
cargo build
```
```
cargo build --release
```

### Windows

On Windows you can download [precompiled OpenSSL binaries](http://slproweb.com/products/Win32OpenSSL.html) (non-light, 64bit). You build with
```
OPENSSL_DIR=C:\OpenSSL-WIN64 OPENSSL_LIBS=libssl_static:libcrypto_static cargo build
```

### Static Linking

Static linking of OpenSSL is done by prepending the `cargo build` command with `OPENSSL_STATIC=1`.
