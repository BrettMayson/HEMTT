# Scripts

Scripts can be called from the command line using `hemtt script <script>`.

The files are located in the `.hemtt/scripts` folder, and are written in [Rhai](../index.md).

They have access to all the same [libraries](../library/index.md) as [hooks](../hooks.md), but only use the [real file system](../library/filesystem.md#hemtt_rfs---real-file-system), since they run outside of the build process.

Scripts are useful for automating tasks that are not part of the build process, such as creating a new addon, or updating the version number.

## Calling from Hooks

Scripts can be called from other scripts or from hooks using `HEMTT.script(<script>)`. The script will still only have access to the real file system.

The script can return a value that will be passed back to the hook.

**.hemtt/scripts/value.rhai**

```js
1 + 1 * 2
```

**.hemtt/hooks/post_release/print_value.rhai**

```js
let value = HEMTT.script("value");
if value != 3 {
    fatal("Value is not 3");
}
println(value)
```
