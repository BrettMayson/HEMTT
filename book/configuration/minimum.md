# Minimum Configuration

The minimum configuration only requires a `name` and `prefix` to be set.

```toml,fp=.hemtt/project.toml
name = "Advanced Banana Environment"
prefix = "abe"
```

## name

The name of your mod, currently unused.

## prefix

The prefix of your mod.

It should be used in the `$PBOPREFIX$` file in each addon folder in the following format.

```text,fp=addons/main/$PBOPREFIX$
z\{prefix}\addons\main
```

```text,fp=addons/main/$PBOPREFIX$
z\abe\addons\main
```
