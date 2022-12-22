# hemtt release

**This command is not yet ready for use.**

<code>
hemtt release
</code>

`hemtt release` will build your mod into `.hemtt/release`. It will create `bisign` files for all addons, and a `bikey` for validation.

It is intended to be used for releasing your mod.

## Configuration

`hemtt release` is built the same way as `hemtt build`, and will use its configuration.

**hemtt.toml**

```toml
[hemtt.release]
```
