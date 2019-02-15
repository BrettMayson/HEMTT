# HEMTT
Build System for Arma 3 powered by [armake2](https://github.com/KoffeinFlummi/armake2) for Linux and Windows.

[Example Project](https://github.com/synixebrett/HEMTT-Example)

### THIS IS PROJECT IS IN BETA

## Using HEMTT
HEMTT is a CLI tool that must be called from the root of your project. HEMTT either needs to be placed in the root and called with `./hemtt`.

## Creating a HEMTT Project

You can either create a brand new Arma 3 mod by using `hemtt create` while in an empty directory or use `hemtt init` while in an existing mod folder to create a `hemtt.json` file.
The `hemtt.json` file keeps track various properties of your project.

## Creating a new addon

To add an additional addon to your project with all the skeleton files you need, use `./hemtt addon [name]`

## Building

You can create a build by using `hemtt build` or a release build by using `hemtt build --release`.
Any files you want to be included in the `releases` folder must be included in `files` in the `hemtt.json` file.
HEMTT will use `addons/main/script_version.hpp` to get version information when doing a release build.

## Notes

It is recommended you add the following to your .gitignore if you are putting your mod on GitHub
```
keys/
releases/
tools/
```