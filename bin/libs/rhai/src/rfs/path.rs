use rhai::plugin::*;

#[allow(clippy::ptr_arg)]
#[export_module]
pub mod path_functions {
    use std::path::PathBuf;

    #[rhai_fn(global, pure)]
    pub fn join(path: &mut PathBuf, other: &str) -> PathBuf {
        path.join(other)
    }

    #[rhai_fn(global, pure, get = "exists")]
    pub fn exists(path: &mut PathBuf) -> bool {
        path.exists()
    }

    #[rhai_fn(global, pure, get = "is_dir")]
    pub fn is_dir(path: &mut PathBuf) -> bool {
        path.is_dir()
    }

    #[rhai_fn(global, pure, get = "is_file")]
    pub fn is_file(path: &mut PathBuf) -> bool {
        path.is_file()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(path: &mut PathBuf) -> String {
        path.display().to_string()
    }
}
