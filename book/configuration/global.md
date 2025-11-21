# Global Configuration

HEMTT can optionally have a global configuration file.

## Location

In order of precedence, the global configuration file can be located at:

| Platform | Location                                      |
|----------|-----------------------------------------------|
| Windows  | `%APPDATA%\hemtt\config.toml`                |
| Linux    | `$XDG_CONFIG_HOME/hemtt/config.toml`<br>`~/.config/hemtt/config.toml` |
| macOS    | `~/Library/Application Support/hemtt/config.toml` |

## Usage

Currently, the global configuration file is only utilized by the `launch` command to define launch profiles and pointers.

Read more about the global configuration in the [launch command documentation](../commands/launch.md#global-configuration).
