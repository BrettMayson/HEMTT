use hemtt_version::Version;
use rhai::plugin::{
    export_module, Dynamic, FnAccess, FnNamespace, Module, NativeCallContext, PluginFunction,
    RhaiResult, TypeId,
};

use crate::context::Context;

use super::project::RhaiProject;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RhaiHemtt {
    version: Version,
    project: RhaiProject,
    folder: String,
}

impl RhaiHemtt {
    pub fn new(ctx: &Context) -> Self {
        Self {
            version: Version::try_from(env!("CARGO_PKG_VERSION")).unwrap(),
            project: RhaiProject::new(ctx),
            folder: ctx.folder().to_string(),
        }
    }
}

#[export_module]
pub mod project_functions {
    #[rhai_fn(global, pure)]
    pub fn version(hemtt: &mut RhaiHemtt) -> Version {
        hemtt.version.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn project(hemtt: &mut RhaiHemtt) -> RhaiProject {
        hemtt.project.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn mode(hemtt: &mut RhaiHemtt) -> String {
        hemtt.folder.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn is_dev(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "dev"
    }

    #[rhai_fn(global, pure)]
    pub fn is_launch(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "launch"
    }

    #[rhai_fn(global, pure)]
    pub fn is_build(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "build"
    }

    #[rhai_fn(global, pure)]
    pub fn is_release(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "release"
    }
}
