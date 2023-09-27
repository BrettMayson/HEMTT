use rhai::plugin::{
    export_module, Dynamic, FnAccess, FnNamespace, Module, NativeCallContext, PluginFunction,
    RhaiResult, TypeId,
};

#[export_module]
pub mod version_functions {
    use hemtt_common::version::Version;

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn to_string(version: &mut Version) -> String {
        version.to_string()
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn to_string_short(version: &mut Version) -> String {
        format!(
            "{}.{}.{}",
            version.major(),
            version.minor(),
            version.patch()
        )
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn major(version: &mut Version) -> i64 {
        i64::from(version.major())
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn minor(version: &mut Version) -> i64 {
        i64::from(version.minor())
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn patch(version: &mut Version) -> i64 {
        i64::from(version.patch())
    }

    #[rustversion::attr(since(1.73), allow(clippy::needless_pass_by_ref_mut))]
    #[rhai_fn(global, pure)]
    pub fn build(version: &mut Version) -> i64 {
        version.build().map(i64::from).unwrap_or_default()
    }
}
