# Examples

## Bumping the minor version

```js,fp=.hemtt/hooks/pre_release/bump_minor.rhai
// Read the current contents of script_version.hpp
let script_version = HEMTT_RFS.join("addons")
    .join("main")
    .join("script_version.hpp")
    .open_file()
    .read();

// Replace the current version with the new version
let prefix = "#define MINOR ";
let current = HEMTT.project().version().minor();
let next = current + 1;

script_version.replace(prefix + current.to_string(), prefix + next.to_string());

// Write the modified contents to script_version.hpp
HEMTT_RFS.join("addons")
    .join("main")
    .join("script_version.hpp")
    .create_file()
    .write(script_version);
```
