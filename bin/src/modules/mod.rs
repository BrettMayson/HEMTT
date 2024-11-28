use crate::{context::Context, error::Error, report::Report};

mod binarize;
mod file_patching;
mod files;
mod new;
mod rapifier;
mod sqf;
mod stringtables;

pub mod archive;
pub mod bom;
pub mod hook;
pub mod pbo;
pub(crate) mod sign;

pub use binarize::Binarize;
pub use file_patching::FilePatching;
pub use files::Files;
pub use hook::Hooks;
pub use new::Licenses;
pub use rapifier::{AddonConfigs, Rapifier};
pub use sign::Sign;
pub use sqf::SQFCompiler;
pub use stringtables::Stringtables;

pub trait Module {
    fn name(&self) -> &'static str;
    /// Executes the module's `init` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn init(&mut self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
    /// Executes the module's `check` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn check(&self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
    /// Executes the module's `pre_build` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn pre_build(&self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
    /// Executes the module's `post_build` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn post_build(&self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
    /// Executes the module's `pre_release` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn pre_release(&self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
    /// Executes the module's `post_release` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn post_release(&self, _ctx: &Context) -> Result<Report, Error> {
        Ok(Report::new())
    }
}
