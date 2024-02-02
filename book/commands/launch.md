# hemtt launch

<pre><code>Launch Arma 3 with your mod and dependencies.

Usage: hemtt launch [OPTIONS] [config] [-- &lt;passthrough&gt;...]

Arguments:
  [config]
        Launches with the specified `hemtt.launch.<config>` configuration

  [passthrough]...
        Passthrough additional arguments to Arma 3

Options:
    <a href="#-e---executable">-e, --executable &lt;executable&gt;</a>
        Arma 3 executable to launch

    <!-- <a href="#-S---with-server">-S, --with-server</a>
        Launches a dedicated server alongside the client -->

    <a href="#-i---instances">-i, --instances &lt;instances&gt;</a>
          Launches multiple instances of the game

          [default: 1]

    <a href="#-Q---quick">-Q, --quick</a>
        Skips the build step, launching the last built version

    <a href="dev.md#-b---binarize">-b, --binarize</a>
        Use BI's binarize on supported files

    <a href="dev.md#-o---optional">-o, --optional &lt;optional&gt;</a>
        Include an optional addon folder

    <a href="dev.md#-o---all-optionals">-O, --all-optionals</a>
        Include all optional addon folders

    <a href="index.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="index.md#-v">-v...</a>
        Verbosity level
        

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt launch` is used to build and launch a dev version of your mod. It will run the [`hemtt dev`](dev.md) command internally after a few checks, options are passed to the `dev` command.

## Configuration

`hemtt launch` requires the [`mainprefix`](../configuration/index.md#main-prefix) option to be set.

**.hemtt/project.toml**

```toml
mainprefix = "z"

# Launched with `hemtt launch`
[hemtt.launch.default]
workshop = [
    "450814997", # CBA_A3's Workshop ID
]
presets = [
    "main", # .html presets from .hemtt/presets/
]
dlc = [
    "Western Sahara",
]
optionals = [
    "caramel",
]
parameters = [
    "-skipIntro",           # These parameters are passed to the Arma 3 executable
    "-noSplash",            # They do not need to be added to your list
    "-showScriptErrors",    # You can add additional parameters here
    "-debug",
    "-filePatching",
    "Path\\To\\mission.sqm", # Launch into existing Editor Mission - \\ needed
]
executable = "arma3" # Default: "arma3_x64"

# Launched with `hemtt launch vn`
[hemtt.launch.vn]
workshop = [
    "450814997", # CBA_A3's Workshop ID
]
dlc = [
    "S.O.G. Prairie Fire",
]
```

### workshop

A list of workshop IDs to launch with your mod. These are not subscribed to, and will need to be manually subscribed to in Steam.

### presets

A list of `.html` presets to launch with your mod. Exported from the Arma 3 Launcher, and kept in `.hemtt/presets/`.

### dlc

A list of DLCs to launch with your mod. The fullname or short-code can be used.

Currently supported DLCs:

| Full Name           | Short Code |
| ------------------- | ---------- |
| Contact             | contact    |
| Global Mobilization | gm         |
| S.O.G. Prairie Fire | vn         |
| CSLA Iron Curtain   | csla       |
| Western Sahara      | ws         |
| Spearhead 1944      | spe        |

### optionals

A list of optional addon folders to launch with your mod.

### parameters

A list of [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters) to pass to the Arma 3 executable.

### executable

The name of the Arma 3 executable to launch. This is usually `arma3` or `arma3_x64`. Do not include the `.exe` extension, it will be added automatically on Windows. Only paths relative to the Arma 3 directory are supported.

## Options

<!-- ### -S, --with-server

Launches a dedicated server alongside the client.

```bash
hemtt launch -S
``` -->

### -i, --instances &lt;instances&gt;

Launches multiple instances of the game. If unspecified, it will default to 1.

```bash
hemtt launch -i 2
```

### -Q, --quick

Skips the build step, launching the last built version.
Will throw an error if no build has been made, or no symlink exists.

```bash
hemtt launch -Q
```

### -e, --executable &lt;executable&gt;

The Arma 3 executable to launch. Overrides the `executable` option in the configuration file.

```bash
hemtt launch -e arma3profiling_x64 # Relative to the Arma 3 directory
hemtt launch -e "C:\Program Files\Steam\steamapps\common\Arma 3\arma3_x64.exe" # Absolute path
```

## Passthrough Options

Any options after `--` will be passed to the Arma 3 executable. This is useful for passing additional [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters).

```bash
hemtt launch -- -world=empty -window
```
