# Hook Examples

## Renaming the release zip

We want to modify the release zip to a different name format, we need to use the real file system during the `post_release` phase. This means that the hook will only run during the [hemtt release](../../commands/release.md) command.

**.hemtt/hooks/post_release/rename_zip.rhai**

```ts
let releases = HEMTT_RFS.join("releases");
let src = releases.join(HEMTT.project().prefix() + "-latest.zip"); // "prefix-latest.zip"
let dst = releases.join("@" + HEMTT.project().prefix() + ".zip"); // "@prefix.zip"
if src.is_file() { // support --no-archive
    print("Moving zip to " + dst);
    if !src.move(dst) {
        fatal("Failed to move " + src + " to " + dst);
    }
}
```

## Setting the version in a file

We want to set the version of the project in the `mod.cpp` file included in our builds, we need to use the virtual file system during the `pre_build` phase. This means that the hook will run during the [hemtt build](../../commands/build.md) and [hemtt release](../../commands/release.md) commands.

Since we are using the virtual file system, the file on disk will not be modified.

**.hemtt/hooks/pre_build/set_version.rhai**

```ts
let modcpp = HEMTT_VFS.join("mod.cpp").open_file().read(); // Read the contents of mod.cpp
modcpp.replace("0.0.0", HEMTT.project().version().to_string_short()); // Replace the placeholder version with the actual version
HEMTT_VFS.join("mod.cpp").create_file().write(modcpp); // Write the new contents over the old contents
```
