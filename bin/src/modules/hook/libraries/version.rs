use rhai::plugin::{
    export_module, Dynamic, FnNamespace, FuncRegistration, Module, NativeCallContext, PluginFunc,
    RhaiResult, TypeId,
};

#[allow(clippy::needless_pass_by_ref_mut)]
#[export_module]
pub mod version_functions {
    use hemtt_common::version::Version;

    #[rhai_fn(global, pure)]
    pub fn to_string(version: &mut Version) -> String {
        version.to_string()
    }

    #[rhai_fn(global, pure)]
    pub fn to_string_short(version: &mut Version) -> String {
        format!(
            "{}.{}.{}",
            version.major(),
            version.minor(),
            version.patch()
        )
    }

    #[rhai_fn(global, pure)]
    pub fn major(version: &mut Version) -> i64 {
        i64::from(version.major())
    }

    #[rhai_fn(global, pure)]
    pub fn minor(version: &mut Version) -> i64 {
        i64::from(version.minor())
    }

    #[rhai_fn(global, pure)]
    pub fn patch(version: &mut Version) -> i64 {
        i64::from(version.patch())
    }

    #[rhai_fn(global, pure)]
    pub fn build(version: &mut Version) -> i64 {
        version.build().map(i64::from).unwrap_or_default()
    }
}
