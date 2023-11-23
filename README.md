# HEMTT

An opinionated build system for Arma 3 mods.

[![GitHub Release](https://img.shields.io/github/v/release/brettmayson/hemtt?style=flat-square&label=Latest)](https://github.com/BrettMayson/HEMTT/releases)
[![HEMTT Downloads](https://img.shields.io/github/downloads/BrettMayson/HEMTT/total.svg?style=flat-square&label=Downloads)](https://github.com/BrettMayson/HEMTT/releases)
[![Codecov](https://img.shields.io/codecov/c/github/brettmayson/hemtt?style=flat-square&label=Coverage)](https://app.codecov.io/gh/brettmayson/hemtt)
[![ACE3 Discord](https://img.shields.io/badge/Discord-Join-darkviolet.svg?style=flat-square)](https://acemod.org/discord)

[The HEMTT Book](https://brettmayson.github.io/HEMTT)

## Installation

[![GitHub](https://img.shields.io/badge/GitHub-lightblue.svg?style=flat-square)](https://github.com/BrettMayson/HEMTT/releases)
[![Chocolatey](https://img.shields.io/badge/Chocolatey-lightblue.svg?style=flat-square)](https://community.chocolatey.org/packages/HEMTT)
[![winstall](https://img.shields.io/badge/WinGet-lightblue.svg?style=flat-square)](https://winstall.app/apps/BrettMayson.HEMTT)

[Read it in the book](https://brettmayson.github.io/HEMTT/installation.html)

## Browser Extension

[![Mozilla Add-on Rating](https://img.shields.io/amo/rating/hemtt?style=flat-square&label=Firefox)](https://addons.mozilla.org/en-US/firefox/addon/hemtt/)
[![Chrome Web Store](https://img.shields.io/chrome-web-store/rating/glhbbnnlagbnmkclalflihdnfhbjmmip?style=flat-square&label=Chrome)](https://chromewebstore.google.com/detail/glhbbnnlagbnmkclalflihdnfhbjmmip)

View .paa files directly on GitHub!

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
