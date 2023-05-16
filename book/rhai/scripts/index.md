# Scripts

Scripts can be called from the command line using `hemtt script <script>`.

The files are located in the `.hemtt/scripts` folder, and are written in [Rhai](/rhai/index.md).

They have access to all the same [libraries](/rhai/library/index.md) as [hooks](/rhai/hooks.md), but only use the [real file system](/rhai/library/filesystem.md#hemtt_rfs---real-file-system), since they run outside of the build process.

Scripts are useful for automating tasks that are not part of the build process, such as creating a new addon, or updating the version number.
