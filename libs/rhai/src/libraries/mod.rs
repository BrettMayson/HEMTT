use rhai::{combine_with_exported_module, def_package};

pub mod project;
mod rfs;
pub mod time;
pub mod version;
mod vfs;

def_package! {
    pub RfsPackage(lib) {
        combine_with_exported_module!(lib, "hemtt_rfs_path", rfs::path::path_functions);
        combine_with_exported_module!(lib, "hemtt_rfs_file", rfs::file::file_functions);
    }
}

def_package! {
    pub VfsPackage(lib) {
        combine_with_exported_module!(lib, "hemtt_vfs_path", vfs::path::path_functions);
        combine_with_exported_module!(lib, "hemtt_vfs_file", vfs::file::file_functions);
    }
}
