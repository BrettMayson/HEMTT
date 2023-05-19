use rhai::{combine_with_exported_module, def_package};

pub mod hemtt;
mod project;
mod rfs;
mod version;
mod vfs;

def_package! {
    pub HEMTTPackage(lib) {
        combine_with_exported_module!(lib, "hemtt", hemtt::project_functions);
        combine_with_exported_module!(lib, "hemtt_version", version::version_functions);
        combine_with_exported_module!(lib, "hemtt_project", project::project_functions);
    }
}

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
