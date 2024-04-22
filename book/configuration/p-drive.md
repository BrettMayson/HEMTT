# P Drive

By default, HEMTT does not require a P Drive (sometimes called a Work Drive), and many mods do not require one.

HEMTT supports a P Drive for mods that do require one, but only to access `\a3\`.

## Includes

Whenever possible, an `.\include\` folder should be used in place of a P Drive. Any files placed in the `.\include\` folder can be used from models or scripts as if they were in a P Drive. No `$PBOPREFIX$` is required, but the full path must be created in the `.\include\` folder.

The most common use case is for CBA's script_macros_common.hpp, you can see an example of this in [ACE's GitHub Repo](include/x/cba/addons/main/script_macros_common.hpp).

## Requiring P Drive

If a P Drive is required by the project, it **must** specify as such. If the flag is not set, HEMTT will not allow the P Drive to be used.

**.hemtt/project.toml**

```toml
[hemtt.build]
pdrive = "require"
```

When required by the project, HEMTT will fail to build the project if all required files can not be resolved. HEMTT will only enable use of `P:\a3\`.

HEMTT will look for a P Drive in the following order:

### P Drive (Mounted)

HEMTT will use P:\ as expected when it exists.

### P Drive (Unmounted)

HEMTT will use the path configured in Arma 3 Tools as the P Drive, even if it is customized from the default and unmounted.

### Arma 3 Installation

If no P Drive, mounted or unmounted, is found, HEMTT will attempt to extract the required files from your Arma 3 Installation.
