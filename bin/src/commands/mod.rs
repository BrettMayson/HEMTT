pub mod book;
pub mod build;
pub mod check;
pub mod clean;
pub mod dev;
pub mod doctor;
pub mod explain;
pub mod keys;
pub mod launch;
pub mod license;
pub mod link;
pub mod localization;
pub mod manage;
pub mod new;
pub mod publish;
pub mod release;
pub mod script;
pub mod utils;
pub mod value;
pub mod wiki;

#[cfg(windows)]
pub mod photoshoot;

/// Adds modules that should apply to:
/// - hemtt check
/// - hemtt dev
/// - hemtt build
/// - hemtt release
pub fn global_modules(executor: &mut crate::executor::Executor) {
    executor.add_module(Box::<crate::modules::bom::BOMCheck>::default());
    executor.add_module(Box::<crate::modules::fnl::FineNewLineCheck>::default());
    executor.add_module(Box::<crate::modules::git::Git>::default());
    executor.add_module(Box::<crate::modules::Hooks>::default());
    executor.add_module(Box::<crate::modules::Stringtables>::default());
    executor.add_module(Box::<crate::modules::SQFCompiler>::default());
    executor.add_module(Box::<crate::modules::PboPrefix>::default());
}

/// Check that every addon named with `--just` exists, a typo would otherwise
/// build nothing and still succeed
///
/// # Errors
/// [`Error::Addon`] if an addon is not found
pub fn check_just(ctx: &crate::context::Context, just: &[String]) -> Result<(), crate::Error> {
    for name in just {
        if !ctx
            .addons()
            .iter()
            .any(|a| a.name().eq_ignore_ascii_case(name))
        {
            return Err(crate::Error::Addon(
                hemtt_workspace::addons::Error::AddonNotFound(name.clone()),
            ));
        }
    }
    Ok(())
}

#[derive(clap::Args, Debug)]
pub struct JustArgs {
    #[arg(long, action = clap::ArgAction::Append)]
    /// Only build the given addon
    pub(crate) just: Vec<String>,
}
