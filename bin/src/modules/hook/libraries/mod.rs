use rhai::{combine_with_exported_module, def_package};

pub mod hemtt;

def_package! {
    pub HEMTTPackage(lib) {
        combine_with_exported_module!(lib, "hemtt", hemtt::project_functions);
        combine_with_exported_module!(lib, "hemtt_version", hemtt_rhai::libraries::version::version_functions);
        combine_with_exported_module!(lib, "hemtt_project", hemtt_rhai::libraries::project::project_functions);
    }
}
