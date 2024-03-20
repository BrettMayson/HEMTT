use hemtt_common::version::Version;
use rhai::plugin::{
    export_module, Dynamic, FnNamespace, FuncRegistration, Module, NativeCallContext, PluginFunc,
    RhaiResult, TypeId,
};

use crate::context::Context;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RhaiProject {
    name: String,
    prefix: String,
    mainprefix: String,
    version: Version,
    // addons: Vec<Addon>,
}

impl RhaiProject {
    pub fn new(ctx: &Context) -> Self {
        Self {
            name: ctx.config().name().to_string(),
            prefix: ctx.config().prefix().to_string(),
            mainprefix: ctx
                .config()
                .mainprefix()
                .map_or_else(String::new, std::string::ToString::to_string),
            version: ctx
                .config()
                .version()
                .get(ctx.workspace().vfs())
                .expect("version config is valid to get to rhai module"),
            // addons: ctx.addons().to_vec(),
        }
    }
}

#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::unwrap_used)] // coming from rhai codegen
#[export_module]
pub mod project_functions {
    use crate::modules::hook::libraries::project::RhaiProject;

    #[rhai_fn(global, pure)]
    pub fn name(project: &mut RhaiProject) -> String {
        project.name.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn prefix(project: &mut RhaiProject) -> String {
        project.prefix.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn mainprefix(project: &mut RhaiProject) -> String {
        project.mainprefix.clone()
    }

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
