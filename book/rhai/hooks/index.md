# Hooks

HEMTT supports hooks at various points in the build process. The hooks are written using [Rhai](https://rhai.rs/). Rhai has an [extension for VSCode](https://marketplace.visualstudio.com/items?itemName=rhaiscript.vscode-rhai) that provides syntax highlighting.

Some example Rhai scripts can be found on the [Rhai Playground](https://rhai.rs/playground/stable/). Additional commands can be requested as a [GitHub Discussion](https://github.com/BrettMayson/HEMTT/discussions/categories/hook-commands).

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

| Hook           | File System                                                      |
| -------------- | ---------------------------------------------------------------- |
| `pre_build`    | [Virtual](library/filesystem.md#hemtt_vfs---virtual-file-system) |
| `post_build`   | [Virtual](library/filesystem.md#hemtt_vfs---virtual-file-system) |
| `pre_release`  | [Real](library/filesystem.md#hemtt_rfs---real-file-system)       |
| `archive`      | [Real](library/filesystem.md#hemtt_rfs---real-file-system)       |
| `post_release` | [Real](library/filesystem.md#hemtt_rfs---real-file-system)       |

### `pre_build`

The `pre_build` hook is run before any preprocessing, binarization, or packing PBOs. This is the place to modify files that will be packed into the PBOs.

### `post_build`

The `post_build` hook is run after all preprocessing, binarization, and packing PBOs. It is run before any release tasks.

### `pre_release`

The `pre_release` hook is run before any release tasks. It is only run during the [hemtt release](../../commands/release.md) command.

### `archive`

The `archive` hook is run after HEMTT is done modifying the PBOs or other files, but before they are archived. If you need to rename or move files, this is the place to do it.

### `post_release`

The `post_release` hook is run after all release tasks, and archives have been created. It is only run during the [hemtt release](../../commands/release.md) command.
