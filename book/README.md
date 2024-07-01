# The HEMTT Book

## What is HEMTT?

HEMTT is used to build your Arma 3 mod into PBOs for development and release.
It is a replacement for tools like [Addon Builder](https://community.bistudio.com/wiki/Addon_Builder) and [pboProject](https://community.bistudio.com/wiki/pboProject).

HEMTT supports the majority of the features found in Bohemia's tools, but prioritizes support for the workflows of best practices and modern modding.

HEMTT is slightly opinionated, most configurations are supported, but some niche features are not and will not be supported. This is to keep the codebase small and maintainable, as well as promote best practices.

## Recommended Workflow

HEMTT works best when used in a Git based project, but the HEMTT executable should not be added to your version control.

It is also recommended to use VSCode for your Arma 3 modding, as it has great extensions that will support you in your modding journey.

Additionally the [HEMTT VSCode extension](https://marketplace.visualstudio.com/items?itemName=BrettMayson.hemtt) is available.

```admonish warning
The VSCode Plugin is in development, and available as an alpha
```

Using HEMTT in VSCode's integrated terminal is the recommended way to use HEMTT, the output can have helpful information that may be missed otherwise.
