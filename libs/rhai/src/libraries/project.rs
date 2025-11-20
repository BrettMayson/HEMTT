use hemtt_common::version::Version;
use hemtt_workspace::addons::Addon;
use rhai::plugin::{
    Dynamic, FnNamespace, FuncRegistration, Module, NativeCallContext, PluginFunc, RhaiResult,
    TypeId, export_module,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RhaiProject {
    pub name: String,
    pub author: String,
    pub prefix: String,
    pub mainprefix: String,
    pub version: Version,
    pub addons: Vec<Addon>,
}

#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::unwrap_used)] // coming from rhai codegen
#[export_module]
pub mod project_functions {
    use super::RhaiProject;

    #[rhai_fn(global, pure)]
    pub fn name(project: &mut RhaiProject) -> String {
        project.name.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn author(project: &mut RhaiProject) -> String {
        project.author.clone()
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

    #[rhai_fn(global, pure)]
    pub fn addons(project: &mut RhaiProject) -> Vec<Addon> {
        project.addons.clone()
    }
}
