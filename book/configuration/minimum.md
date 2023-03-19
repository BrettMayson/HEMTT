# Minimum Configuration

The minimum configuration only requires a `name` and `prefix` to be set.

**.hemtt/project.toml**

```toml
name = "Advanced Banana Environment"
prefix = "abe"
```

## name

The name of your mod, currently unused.

## prefix

The prefix of your mod.

It should be used in the `$PBOPREFIX$` file in each addon folder in the following format.

```text
z\{prefix}\addons\main
```

**addons/main/$PBOPREFIX$**

```text
z\abe\addons\main
```
