# Version Configuration

HEMTT uses the project version as part of the signing authority.
It is also included as an property in built PBOs.

No `[version]` configuration is required if you use macros in your `script_version.hpp` file.

**.hemtt/project.toml**

```toml
[version]
path = "addons/main/script_version.hpp" # Default

major = 1 # Overrides path when set
minor = 0
patch = 4
build = 3 # Optional

git_hash = 0 # Default: 8
```

## Macros

By default, HEMTT will look for `addons/main/script_version.hpp` and use the version components defined there. No `[version]` configuration is required.
A major, minor, and patch version are required, and a build version is optional.

**/addons/main/script_version.hpp**

```cpp
#define MAJOR 1
#define MINOR 0
#define PATCH 4 // `#define PATCHLVL` can also be used
#define BUILD 3 // Optional
```

If your macros are in another file, you can set the path with the `version.path` key.

**.hemtt/project.toml**

```toml
[version]
path = "addons/common/script_version.hpp"
```

## Defined in Configuration

If you do not want to use a version file, you can set the version components directly in the configuration. If a version is defined in the configuration, the macros will not be used, even if a path is set.

**.hemtt/project.toml**

```toml
[version]
major = 1
minor = 0
patch = 4
build = 3 # Optional
```

## Git Hash

By default, HEMTT will include the first 8 characters of the current git hash in the version.
Since the git hash is enabled by default, without configuration HEMTT will require a git repository with at least one commit to be present.
The git hash can be disabled by setting `version.git_hash = 0`, or configured to a different length.

**.hemtt/project.toml**

```toml
[version]
git_hash = 0 # Disabled
git_hash = 4 # 4 characters
```
