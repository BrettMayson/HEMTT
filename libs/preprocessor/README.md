# Preprocessor

## Differences from BI Preprocessor

- The following built-in macros are not supported:
    - `__has_include()`
    - `__GAME_VER__`
    - `__GAME_VER_MAJ__`
    - `__GAME_VER_MIN__`
    - `__GAME_BUILD__`
    - `__A3_DEBUG__`
    - `__EXEC()`
    - `__EVAL()`

- Tabs after `#define` are ignored.
    ```cpp
    #define EXAMPLE				1
    ```
    BI:
    ```cpp
    value =				1;
    ```

    HEMTT:
    ```cpp
    value = 1;
    ```
