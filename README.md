# HEMTT
Build System for Arma 3 powered by armake for Linux and Windows

### THIS IS PROJECT IS IN ALPHA

## Using HEMTT
HEMTT is a CLI tool that must be called from the root of your project. HEMTT either needs to be placed in the root and called with `hemtt` or placed in a directory and called with `[directory]/hemtt`.

## Creating a HEMTT Project

You can either create a brand new Arma 3 mod by using `hemtt create` while in an empty directory or use `hemtt init` while in an existing mod folder to create a `hemtt.json` file.
The `hemtt.json` file keeps track various properties of your project.

## Creating a new addon

To add an additional addon to your project with all the skeleton files you need, use `hemtt addon [name]`

## Building

You can create an inplace build by using `hemtt build` or a release build by using `hemtt build --release`.
Any files you want to be included in the `releases` folder must be included in `files` in the `hemtt.json` file.
HEMTT will use `addons/main/script_version.hpp` to get version information when doing a release build.

## Notes
If you are using an ANSI terminal (Windows CMD) you will notice a lack of colors, and see 24-bit color characters. To hide these use `hemtt -n ...`

It is recommended you add the following to your .gitignore if you are putting your mod on GitHub
```
keys/
releases/
tools/
```

You may also want to add `include/` if you are not adding anything additional into that folder.
