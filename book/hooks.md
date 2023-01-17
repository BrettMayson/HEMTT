# Hooks

HEMTT supports hooks at various points in the build process. The hooks are written using [Rhai](https://rhai.rs/). Rhai has an [extension for VSCode](https://marketplace.visualstudio.com/items?itemName=rhaiscript.vscode-rhai) that provides syntax highlighting.

Some example Rhai scripts can be found on the [Rhai Playground](https://rhai.rs/playground/stable/)

Hooks are stored in the `.hemtt/hooks/{phase}` folders. The `{phase}` is the name of the phase that the hook is run in. The hooks are run in alphabetical order.

**Example**
```
.hemtt
└── hooks
    ├── pre_build
    │   ├── 01_example.rhai
    │   └── 02_example.rhai
    └── post_build
        ├── 01_example.rhai
        └── 02_example.rhai
```

## Phases

There are 4 phases of the build process that can be hooked into:

| Hook | File System |
| --- | --- |
| `pre_build` | [Virtual](#virtual) |
| `post_build` | [Virtual](#virtual) |
| `pre_release` | [Real](#real) |
| `post_release` | [Real](#real) |

### `pre_build`

The `pre_build` hook is run before any preprocessing, binarization, or packing PBOs. This is the place to modify files that will be packed into the PBOs.

### `post_build`

The `post_build` hook is run after all preprocessing, binarization, and packing PBOs. It is run before any release tasks.

### `pre_release`

The `pre_release` hook is run before any release tasks. It is only run during the [hemtt release](commands-release.md) command.

### `post_release`

The `post_release` hook is run after all release tasks, and archives have been created. It is only run during the [hemtt release](commands-release.md) command.

## Constants

Several constants are available to the hook scripts. These are:

| Constant | Description |
| --- | --- |
| `HEMTT_VERSION` | The version of HEMTT |
| `HEMTT_PROJECT_VERSION` | The version of the project, ex: 1.3.5-a8c20d |
| `HEMTT_PROJECT_VERSION_MAJOR` | The major version of the project, ex: 1 |
| `HEMTT_PROJECT_VERSION_MINOR` | The minor version of the project, ex: 3 |
| `HEMTT_PROJECT_VERSION_PATCH` | The patch version of the project, ex: 5 |
| `HEMTT_PROJECT_VERSION_BUILD` | The build of the project, ex: a8c20d |
| `HEMTT_PROJECT_NAME` | The name of the project |
| `HEMTT_PROJECT_PREFIX` | The prefix of the project |
| `HEMTT_PROJECT_HEADERS` | The headers of the project |
| `HEMTT_PROJECT_ADDONS` | The addons of the project |

## File System

### Virtual

`*_build` phases have a virtual file system. This means that the files are not actually written to disk. Files can be read and written to, and will appear only in the build output.

This is useful for modifying files with find-and-replace, or adding files to the build output, without the need for cleaning up after the build.

When using the virtual files system, an additional `HEMTT_VFS` constant is available. It is used as the root path.

**.hemtt/hooks/pre_build/set_version.rhai**

```ts
// Get the path to the script_version.hpp file
let version = HEMTT_VFS
        .join("addons")
        .join("main")
        .join("script_version.hpp");
// Read the current contents
let current = version.open_file().read();
// Replace the placeholder version with the actual version
current.replace("0.0.0", HEMTT_PROJECT_VERSION);
// Write the new contents
// create_file will overwrite the file if it exists
version.create_file().write(current);
```

### Real

`*_release` phases have a real file system. This means that the files are actually written to disk. Be careful when modifying files, as you can modify the project files.

When using the real file system, two additional constants are available. `HEMTT_DIRECTORY` is the root of the project, and `HEMTT_OUTPUT` is the root of the build output.

**.hemtt/hooks/pre_release/set_version.rhai**

```ts
// Read the current contents of the docs/version.txt
// file from the project source
let version = HEMTT_DIRECTORY
        .join("docs")
        .join("version.txt")
        .open_file()
        .read();
// Replace the placeholder version with the actual version
version.replace("0.0.0", HEMTT_PROJECT_VERSION);
// Write the new contents to the build output
// create_file will overwrite the file if it exists
HEMTT_OUTPUT
        .join("docs")
        .join("version.txt")
        .create_file()
        .write(version);
```
