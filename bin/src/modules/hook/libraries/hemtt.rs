use hemtt_common::version::Version;
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
    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn version(hemtt: &mut RhaiHemtt) -> Version {
        hemtt.version.clone()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn project(hemtt: &mut RhaiHemtt) -> RhaiProject {
        hemtt.project.clone()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn mode(hemtt: &mut RhaiHemtt) -> String {
        hemtt.folder.clone()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_dev(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "dev"
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_launch(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "launch"
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_build(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "build"
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn is_release(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "release"
    }
}
