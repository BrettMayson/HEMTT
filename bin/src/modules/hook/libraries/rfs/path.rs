use rhai::plugin::{
    export_module, mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

#[allow(clippy::ptr_arg)]
#[export_module]
pub mod path_functions {
    use std::path::PathBuf;

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn join(path: &mut PathBuf, other: &str) -> PathBuf {
        path.join(other)
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn exists(path: &mut PathBuf) -> bool {
        path.exists()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_dir(path: &mut PathBuf) -> bool {
        path.is_dir()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_file(path: &mut PathBuf) -> bool {
        path.is_file()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(path: &mut PathBuf) -> String {
        path.display().to_string()
    }

    #[rhai_fn(global, name = "copy", pure)]
    pub fn copy(path: &mut PathBuf, other: PathBuf) -> bool {
        if path.is_dir() {
            fs_extra::dir::copy(path, other, &fs_extra::dir::CopyOptions::new()).is_ok()
        } else {
            std::fs::copy(path, other).is_ok()
        }
    }

    #[rhai_fn(global, name = "move", pure)]
    pub fn _move(path: &mut PathBuf, other: PathBuf) -> bool {
        if path.is_dir() {
            fs_extra::dir::move_dir(path, other, &fs_extra::dir::CopyOptions::new()).is_ok()
        } else {
            std::fs::rename(path, other).is_ok()
        }
    }
}
