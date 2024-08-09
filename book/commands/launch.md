# hemtt launch

<pre><code>Launch Arma 3 with your mod and dependencies.

Usage: hemtt launch [OPTIONS] [config]... [-- &lt;passthrough&gt;...]

Arguments:
  [config]...
        Launches with the specified `hemtt.launch.<config>` configurations

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
        
    <a href="build.md#--no-rap">--no-rap</a>
        Do not rapify files

    <a href="--no-filepatching">--no-filepatching</a>
        Do not enable filePatching

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

You can chain multiple configurations together, and they will be overlayed from left to right. Any arrays will be concatenated, and any duplicate keys will be overridden. With the below configuration, `hemtt launch default vn ace` would launch with all three configurations. Note that `default` must be specified when specifying additional configurations, `default` is only implied when no configurations are specified.

## Configuration

`hemtt launch` requires the [`mainprefix`](../configuration/index.md#main-prefix) option to be set.

Launch configurations can be stored in either `.hemtt/project.toml` under `hemtt.launch`, or in a separate file under `.hemtt/launch.toml`. The latter is useful for keeping your main configuration file clean. When using `launch.toml`, the `hemtt.launch` key is not required.

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
mission = "test.VR" # Mission to launch directly into the editor with
parameters = [
    "-skipIntro",           # These parameters are passed to the Arma 3 executable
    "-noSplash",            # They do not need to be added to your list
    "-showScriptErrors",    # You can add additional parameters here
    "-debug",
    "-filePatching",
]
cli_options = [ # CLI options for launch can be set here as part of launch config
    "--quick",              # These parameters are passed to hemtts 'launch' command
    "--expsqfc",            # They do not need to be added to your list
    "--no-filepatching",
    "--binarize",
    "--no-rap",
    "--optional=some_optional",
    "--all-optionals",
]
executable = "arma3" # Default: "arma3_x64"

# Launched with `hemtt launch vn`
[hemtt.launch.vn]
extends = "default"
dlc = [
    "S.O.G. Prairie Fire",
]

# Launched with `hemtt launch ace`
[hemtt.launch.ace]
extends = "default"
workshop = [
    "463939057", # ACE3's Workshop ID
]
```

**.hemtt/launch.toml**

```toml
[default]
workshop = [
    "450814997", # CBA_A3's Workshop ID
]

[vn]
extends = "default"
dlc = [
    "S.O.G. Prairie Fire",
]
```

### extends

The name of another configuration to extend. This will merge all arrays with the base configuration, and override any duplicate keys.

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
| Reaction Forces     | rf         |

### optionals

A list of optional addon folders to launch with your mod.

### mission

The mission to launch directly into the editor with. This can be specified as either the name of a folder in `.hemtt/missions/` (e.g., `test.VR` would launch `.hemtt/missions/test.VR/mission.sqm`) or the relative (to the project root) path to a `mission.sqm` file or a folder containing it.

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

### --no-filepatching

Do not launch Arma 3 with `-filePatching`.

## Passthrough Options

Any options after `--` will be passed to the Arma 3 executable. This is useful for passing additional [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters).

```bash
hemtt launch -- -world=empty -window
```
