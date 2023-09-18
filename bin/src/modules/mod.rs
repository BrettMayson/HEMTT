use crate::{context::Context, error::Error};

pub mod archive;
pub mod hook;
pub mod pbo;

pub use hook::Hooks;

#[cfg(not(target_os = "macos"))]
mod asc;
mod binarize;
mod file_patching;
mod files;
mod lint;
mod new;
mod rapifier;
mod sign;

#[cfg(not(target_os = "macos"))]
pub use asc::ArmaScriptCompiler;
pub use binarize::Binarize;
pub use file_patching::FilePatching;
pub use files::Files;
pub use lint::Lint;
pub use new::Licenses;
pub use rapifier::Rapifier;
pub use sign::Sign;

pub trait Module {
    fn name(&self) -> &'static str;
    /// Executes the module's `init` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn init(&mut self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    /// Executes the module's `check` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn check(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    /// Executes the module's `pre_build` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn pre_build(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    /// Executes the module's `post_build` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn post_build(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    /// Executes the module's `pre_release` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn pre_release(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    /// Executes the module's `post_release` phase
    ///
    /// # Errors
    /// Any error that the module encounters
    fn post_release(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
}
