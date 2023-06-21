# HEMTT

An opinionated build system for Arma 3 mods.

<a href="https://github.com/BrettMayson/HEMTT/releases">
    <img alt="GitHub release (latest SemVer)" src="https://img.shields.io/github/v/release/brettmayson/hemtt?style=flat-square&label=Latest">
    <img src="https://img.shields.io/github/downloads/BrettMayson/HEMTT/total.svg?style=flat-square&label=Downloads" alt="HEMTT Downloads">
</a>
<a href="https://app.codecov.io/gh/brettmayson/hemtt">
    <img alt="Codecov" src="https://img.shields.io/codecov/c/github/brettmayson/hemtt?style=flat-square&label=Coverage">
</a>
<a href="https://acemod.org/discord">
    <img src="https://img.shields.io/badge/Discord-Join-darkviolet.svg?style=flat-square" alt="ACE3 Discord">
</a>

[The HEMTT Book](https://brettmayson.github.io/HEMTT)

## Installation

[Read it in the book](https://brettmayson.github.io/HEMTT/installation.html)

## Example GitHub Actions Workflow

```yaml
name: Build

on: [push]

jobs:
    build:
        runs-on: ubuntu-latest
        steps:
            - uses: actions/checkout@v2
            - name: Setup HEMTT
              uses: arma-actions/hemtt@v1
            - name: Run HEMTT build
              run: hemtt release
            - name: Upload Release
              uses: actions/upload-artifact@v2
              with:
                  name: my-mod-latest
                  path: release/my-mod-latest.zip
```
