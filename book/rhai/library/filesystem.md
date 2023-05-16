# File System

HEMTT has two types file systems, which one is used depends on the context.

[Scripts](../scripts.md) always use the real file system, as the run outside of the build process.

[Hooks](../hooks.md) use the virtual file system during the `pre_build` and `post_build` phases, and the real file system during the `pre_release` and `post_release` phases.

## `HEMTT_VFS` - Virtual File System

`*_build` phases have a virtual file system. This means that the files are not actually written to disk. Files can be created, deleted, read from, written to, and these changes will appear only in the build output.

This is useful for modifying files with find-and-replace, or adding files to the build output, without the need for cleaning up after the build.

When using the virtual files system, the `HEMTT_VFS` constant is available. It is used as the root path.

**.hemtt/project.toml**

```toml
[version]
major = 1
minor = 0
patch = 3
```

**.hemtt/hooks/pre_build/set_version.rhai**

```ts
// Get the path to the script_version.hpp file
let version = HEMTT_VFS
        .join("addons")
        .join("main")
        .join("script_version.hpp");
// Create (or overwrite) the file
let out = version.create_file();
out.write("#define MAJOR " + HEMTT.project().version().major() + "\n");
out.write("#define MINOR " + HEMTT.project().version().minor() + "\n");
out.write("#define PATCH " + HEMTT.project().version().patch() + "\n");
if HEMTT.project().version().build() != "" {
    out.write("#define BUILD " + HEMTT.project().version().build() + "\n");
}
print("Set version to " + HEMTT.project().version().to_string());
```

## `HEMTT_RFS` - Real File System

`*_release` phases have a real file system. This means that the files are actually written to disk.

```admonish danger
Be careful when modifying files while using the real file system, as you can destructively modify the project files. It is recommended to use the virtual file system whenever possible, and commit the changes to the project files prior to testing hooks.
```

When using the real file system, two additional constants are available. `HEMTT_RFS` is the root of the project, and `HEMTT_OUT` is the root of the build output.

**.hemtt/hooks/pre_release/set_version.rhai**

```ts
// Read the current contents of the docs/version.txt
// file from the project source
let version = HEMTT_RFS
        .join("docs")
        .join("version.txt")
        .open_file()
        .read();
// Replace the placeholder version with the actual version
version.replace("0.0.0", HEMTT.project().version().to_string());
// Write the new contents to the build output
// create_file will overwrite the file if it exists
HEMTT_OUT
        .join("docs")
        .join("version.txt")
        .create_file()
        .write(version);
```

# Functions

All the functions below are available on both the virtual and real file systems.

## `join(string)`

Joins the path with the given string.

```ts
HEMTT_VFS.join("addons") // Points to ./addons in the project folder
HEMTT_VFS.join("addons").join("main") // Points to ./addons/main in the project folder
```

## `exists()`

Returns `true` if the path exists.

```ts
HEMTT_VFS.join("addons").exists() // true
HEMTT_VFS.join(".hemtt").join("project.toml").exists() // true
```

## `is_dir()`

Returns `true` if the path is a directory.

```ts
HEMTT_VFS.join("addons").is_dir() // true
HEMTT_VFS.join(".hemtt").join("project.toml").is_dir() // false
```

## `is_file()`

Returns `true` if the path is a file.

```ts
HEMTT_VFS.join("addons").is_file() // false
HEMTT_VFS.join(".hemtt").join("project.toml").is_file() // true
```

## `copy(path)`

Copies the file or directory to the given path.

```ts
HEMTT_VFS.join("docs").copy(HEMTT_OUT.join("docs")) // Copies the docs folder to the build output
```

## `move(path)`

Moves the file or directory to the given path.

```ts
HEMTT_VFS.join("docs").move(HEMTT_OUT.join("docs")) // Moves the docs folder to the build output
```

## `open_file()`

Opens the file for reading.

```ts
HEMTT_VFS.join("docs").join("readme.md").open_file(); // Returns a File object
```

## `create_file()`

Creates the file for writing. Overwrites the file if it exists.

```ts
HEMTT_VFS.join("docs").join("readme.md").create_file(); // Returns a File object
```

## `remove_file()`

Removes the file.

```ts
HEMTT_VFS.join("docs").join("readme.md").remove_file(); // Removes the file
```

## `read()`

Reads the contents of the file.

```ts
HEMTT_VFS.join("docs").join("readme.md").open_file().read(); // Returns a string containing the contents of the file
```

## `write(string)`
Writes the string to the file. Can be called multiple times to append to the file.

```ts
HEMTT_VFS.join("docs").join("readme.md").create_file().write("Hello World!"); // Writes "Hello World!" to the file
```
