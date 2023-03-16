use rhai::plugin::{
    export_module, mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

#[export_module]
pub mod path_functions {
    use rhai::EvalAltResult;
    use vfs::VfsPath;

    #[rhai_fn(global, pure, return_raw)]
    pub fn join(path: &mut VfsPath, other: &str) -> Result<VfsPath, Box<EvalAltResult>> {
        path.join(other).map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, pure, get = "exists", return_raw)]
    pub fn exists(path: &mut VfsPath) -> Result<bool, Box<EvalAltResult>> {
        path.exists().map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, pure, get = "is_dir", return_raw)]
    pub fn is_dir(path: &mut VfsPath) -> Result<bool, Box<EvalAltResult>> {
        path.is_dir().map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, pure, get = "is_file", return_raw)]
    pub fn is_file(path: &mut VfsPath) -> Result<bool, Box<EvalAltResult>> {
        path.is_file().map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(path: &mut VfsPath) -> String {
        path.as_str().to_string()
    }
}
