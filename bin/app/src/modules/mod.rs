use crate::{context::Context, error::Error};

pub mod pbo;

mod binarize;
mod preprocessor;

pub use binarize::Binarize;
pub use preprocessor::Preprocessor;

pub trait Module {
    fn name(&self) -> &'static str;
    fn init(&mut self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    fn check(&self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    fn post_build(&self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    fn pre_release(&self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
    fn post_release(&self, ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
}
