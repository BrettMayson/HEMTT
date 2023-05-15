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
}

impl RhaiHemtt {
    pub fn new(ctx: &Context) -> Self {
        Self {
            version: Version::try_from(env!("CARGO_PKG_VERSION")).unwrap(),
            project: RhaiProject::new(ctx),
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
}
