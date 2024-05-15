# Custom Commands

```admonish danger
You are reading about an advanced and niche feature. This is specific for projects using Intercept and injecting custom SQF commands. This is not a feature that is commonly used.
```

Custom commands are defined using yaml inside the `.hemtt/commands` directory. The file name should be the name of the file, but this is not enforced. The command name will be taken from the defined name in the yaml file.

For reference, you can find all of Arma 3's commands in the [arma3-wiki](https://github.com/acemod/arma3-wiki/blob/dist/commands) repo on the `dist` branch, under the acemod organization.

```yaml
name: bananize
description: Bananize the player, forcing them to only throw bananas.
syntax:
- call: !Unary player
  ret:
    - Boolean
    - The success of the bananization
  params:
  - name: unit
    description: The unit to bananize
    type: Object
argument_loc: Global
effect_loc: Global
examples:
- <sqf>bananize player</sqf>
```
