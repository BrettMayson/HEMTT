use hemtt_common::version::Version;
use rhai::plugin::{
    export_module, Dynamic, FnAccess, FnNamespace, Module, NativeCallContext, PluginFunction,
    RhaiResult, TypeId,
};

use crate::context::Context;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RhaiProject {
    name: String,
    prefix: String,
    version: Version,
    // addons: Vec<Addon>,
}

impl RhaiProject {
    pub fn new(ctx: &Context) -> Self {
        Self {
            name: ctx.config().name().to_string(),
            prefix: ctx.config().prefix().to_string(),
            version: ctx.config().version().get(ctx.workspace().vfs()).unwrap(),
            // addons: ctx.addons().to_vec(),
        }
    }
}

#[export_module]
pub mod project_functions {
    use crate::modules::hook::libraries::project::RhaiProject;

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn name(project: &mut RhaiProject) -> String {
        project.name.clone()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn prefix(project: &mut RhaiProject) -> String {
        project.prefix.clone()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn version(project: &mut RhaiProject) -> Version {
        project.version.clone()
    }

    // TODO: Add functions to addons
    // #[rhai_fn(global, pure)]
    // pub fn addons(project: &mut RhaiProject) -> Vec<Addon> {
    //     project.addons.clone()
    // }
}
