# Config

HEMTT will provide warnings for common issues in your config, in both the preprocessing and rapifying stages.

## Preprocessor Warnings

### [PW1] Redefine Macro

This warning is emitted when a macro is defined more than once.

```cpp
#define FOO 1
#define FOO 2
```

It may also appear when a macro is defined in a file that is included more than once.

```cpp
// foo.hpp
#define FOO 1

// bar.hpp
#include "foo.hpp"
#include "foo.hpp"
```

### [PW2] Invalid Config Case

This warning is emitted when `config.cpp` is not all lowercase, e.g. `Config.cpp`.

## Rapify Warnings

### [CW1] Parent Case Mismatch

This warning is emitted when an inherited class does not match the case of the defined class.

```cpp
class Parent;
class Child: parent {};
```

### [CW2] CfgMagazineWells was not found in CfgMagazines

This warning is emitted when a `CfgMagazineWells` entry is not found in `CfgMagazines`.

In this example, `abe_plantain` is not found in `CfgMagazines`, and a warning is emitted.

```admonish note title=""
Only entries that start with the project's [prefix](../configuration/index.md#minimum-configuration). No warning will be emitted for `external_banana`.
```

```cpp
class CfgMagazineWells {
    class abe_banana_shooter {
        ADDON[] = {
            "abe_cavendish",
            "abe_plantain"
            "external_banana"
        };
    };
};
class CfgMagazines {
    class abe_cavendish { ... };
};
```
