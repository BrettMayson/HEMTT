# Scripts
Scripts are used to extend the build process of your project.

# Defining a Script
Scripts use a list of shell commands. This small snippet is all that is needed to define a basic script. This example would be ran with `hemtt run build`.
```toml
[scripts.build]
steps = [
    "make all"
]
```
### steps_windows / steps_linux
`steps_windows` and `steps_linux` can be used to run different steps on the respective platforms.
```toml
[scripts.build]
steps_linux = [
    "make linux"
]
steps_windows = [
    "make windows"
]
```

### show_output
All output is hidden by default. Setting `show_output` will display the command being executed and its output.
```toml
[scripts.example]
steps = [
    "echo 'this is an example'"
]
show_output = true
```

### only_development / only_release
Scripts run during normal and release builds. Setting `only_release` will run the script only in release build and setting `only_development` will run the script only in development build.
```toml
[scripts.example]
steps = [
    "echo 'this is an example'"
]
only_release = true
```


# Build Steps
There are 4 different build step definitions. `check`, `prebuild`, `postbuild` and `releasebuild`. These are added to the root of the HEMTT project file. Scripts can be ran using `![script]` and utilities are ran using `@[utility]`. The following example runs the `build` script, the uses `cp` to copy files.
```toml
releasebuild = [
  "!build"
]

[scripts.build]
steps_linux = [
    "make linux",
    "cp bin/ release/{{version}}/ -r"
]
steps_windows = [
    "make windows",
    "copy bin/ release/{{version}}/"
]
```

### foreach
Scripts can be ran for each addons. Inside `check` and `prebuild` the script will be ran for each addon that HEMTT will build, including addons that will be skipped if they are already built. Inside `postbuild` and `releasebuild` only addons that were successfully built with be used, excluding addons that were skipped for being up to date.

In addition to the standard [templating variables](templating.md), additional variables are added when using foreach.

| Variable | check & prebuild    | postbuild & releasebuild |
|----------|---------------------|--------------------------|
| addon    | main                | main                     |
| source   | addons/main         | addons/main              |
| target   | addons/ABE_main.pbo | addons/ABE_main.pbo      |
| time     |                     | (build time in ms)       |

```toml
postbuild = [
    "!buildtime"
]

[scripts.buildtime]
steps = [
    "echo {{addon}} took {{time}} ms to build."
]
show_output = true
foreach = true
```

#### parallel
Requires `foreach` to be true. If a script is thread safe `parallel` can be used to process multiple addons at a time.
