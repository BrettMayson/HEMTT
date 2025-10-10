# File System

HEMTT has two types of file systems, which one is used depends on the context.

[Scripts](../scripts/index.md) always use the real file system, as they run outside of the build process.

[Hooks](../hooks/index.md) use the virtual file system during the `pre_build`, `post_build`, and `pre_release` phases.

## `HEMTT_VFS` - Virtual File System

`pre_build`, `post_build`, and `pre_release` phases have access to the virtual file system. This means that the files are not actually written to disk. Files can be created, deleted, read from, written to, and these changes will appear only in the build output.

This is useful for modifying files with find-and-replace, or adding files to the build output, without the need for cleaning up after the build.

When using the virtual file system, the `HEMTT_VFS` constant is available. It is used as the root path.

```admonish warning
During the `pre_release` phase, only files outside of addons should be changed. PBOs are already built, and changing files inside of addons will have no effect.
```

**.hemtt/project.toml**

```toml
[version]
major = 1
minor = 0
patch = 3
```

**.hemtt/hooks/pre_build/set_version.rhai**

```js
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

All phases and scripts have access to the real file system. This means that the files are actually written to disk.

```admonish danger
Be careful when modifying files while using the real file system, as you can destructively modify the project files. It is recommended to use the virtual file system whenever possible, and commit the changes to the project files prior to testing hooks.
```

When using the real file system, two additional constants are available. `HEMTT_RFS` is the root of the project, and `HEMTT_OUT` is the root of the build output.

**.hemtt/hooks/pre_release/set_version.rhai**

```js
// Read the current contents of the docs/version.txt
// file from the project source
let version = HEMTT_RFS.join("docs").join("version.txt").open_file().read();
// Replace the placeholder version with the actual version
version.replace("0.0.0", HEMTT.project().version().to_string());
// Write the new contents to the build output
// create_file will overwrite the file if it exists
HEMTT_OUT.join("docs").join("version.txt").create_file().write(version);
```

## Functions

All the functions below are available on both the virtual and real file systems.

### `join(string)`

Joins the path with the given string.

```js
HEMTT_VFS.join("addons"); // Points to ./addons in the project folder
HEMTT_VFS.join("addons").join("main"); // Points to ./addons/main in the project folder
```

### `exists()`

Returns `true` if the path exists.

```js
HEMTT_VFS.join("addons").exists(); // true
HEMTT_VFS.join(".hemtt").join("project.toml").exists(); // true
```

### `is_dir()`

Returns `true` if the path is a directory.

```js
HEMTT_VFS.join("addons").is_dir(); // true
HEMTT_VFS.join(".hemtt").join("project.toml").is_dir(); // false
```

### `is_file()`

Returns `true` if the path is a file.

```js
HEMTT_VFS.join("addons").is_file(); // false
HEMTT_VFS.join(".hemtt").join("project.toml").is_file(); // true
```

### `parent()`

Returns the parent directory of the path.  
Will panic if the path is root while using the real file system.  
Will return the root path while using the virtual file system, if already at the root.

```js
HEMTT_VFS.join("addons").parent(); // Points to ./
HEMTT_VFS.join(".hemtt").join("project.toml").parent(); // Points to ./.hemtt
```

### `file_name()`

Returns the file name of the path.

```js
HEMTT_VFS.join("addons").file_name(); // addons
HEMTT_VFS.join(".hemtt").join("project.toml").file_name(); // project.toml
```

### `file_ext()`

Returns the file extension of the path, or an empty string if there is none.

```js
HEMTT_VFS.join("addons").file_ext(); // ""
HEMTT_VFS.join(".hemtt").join("project.toml").file_ext(); // "toml"
```

### `copy(path)`

Copies the file or directory to the given path.

```js
HEMTT_VFS.join("docs").copy(HEMTT_OUT.join("docs")); // Copies the docs folder to the build output
```

### `move(path)`

Moves the file or directory to the given path.

```js
HEMTT_VFS.join("docs").move(HEMTT_OUT.join("docs")); // Moves the docs folder to the build output
```

### `list()`

Lists the contents of the directory. If the path is a file, returns an empty array.

```js
HEMTT_VFS.join("docs").list(); // Returns an array of paths of files and directories in the docs folder
```

### `open_file()`

Opens the file for reading.

```js
HEMTT_VFS.join("docs").join("readme.md").open_file(); // Returns a File object
```

### `create_file()`

Creates the file for writing. Overwrites the file if it exists.

```js
HEMTT_VFS.join("docs").join("readme.md").create_file(); // Returns a File object
```

### `remove_file()`

Removes the file.

```js
HEMTT_VFS.join("docs").join("readme.md").remove_file(); // Removes the file
```

### `read()`

Reads the contents of the file.

```js
HEMTT_VFS.join("docs").join("readme.md").open_file().read(); // Returns a string containing the contents of the file
```

### `write(string)`

Writes the string to the file. Can be called multiple times to append to the file.

```js
HEMTT_VFS.join("docs").join("readme.md").create_file().write("Hello World!"); // Writes "Hello World!" to the file
```

### `create_dir()`

Creates the directory.

```js
HEMTT_VFS.join("docs").create_dir(); // Creates the docs folder
```

### `create_dir_all()`

Creates the directory and all parent directories.

```js
HEMTT_VFS.join("docs").join("images").create_dir_all(); // Creates the images folder and the docs folder if they don't exist
```

### `remove_dir()`

Removes the directory.

```js
HEMTT_VFS.join("docs").remove_dir(); // Removes the docs folder
```

### `remove_dir_all()`

Removes the directory and all its contents.

```js
HEMTT_VFS.join("docs").remove_dir_all(); // Removes the docs folder and all its contents
```
