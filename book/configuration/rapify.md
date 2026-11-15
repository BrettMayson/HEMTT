# Rapify Configuration

The `[rapify]` section in `.hemtt/project.toml` controls options during the rapification process of config files.

```toml,fp=.hemtt/project.toml
[rapify]
add_units = true
add_weapons = ["abe_"]
```

## add_units

Automatically adds matching classes from `CfgVehicles` to the `units` array in `CfgPatches`.

The default is `true`, which matches classes whose names start with the project's `prefix`. Set it to `false` to disable automatic unit registration. A string or array of strings can be used to specify one or more class-name prefixes instead.

Prefix matching is case-insensitive and checks the beginning of each class name.

```toml,fp=.hemtt/project.toml
[rapify]
add_units = true # adds units with the project's prefix
add_units = "abe_" # adds units with the "abe_" prefix
add_units = ["abe_", "other_"] # adds units with the "abe_" and "other_" prefixes
```

## add_weapons

Automatically adds matching classes from `CfgWeapons` to the `weapons` array in `CfgPatches`.

The default is `true`, which matches classes whose names start with the project's `prefix`. Set it to `false` to disable automatic weapon registration. A string or array of strings can be used to specify one or more class-name prefixes instead.

Prefix matching is case-insensitive and checks the beginning of each class name.

```toml,fp=.hemtt/project.toml
[rapify]
add_weapons = true # adds weapons with the project's prefix
add_weapons = "abe_" # adds weapons with the "abe_" prefix
add_weapons = ["abe_", "other_"] # adds weapons with the "abe_" and "other_" prefixes
```
