use crate::{
    context::Context,
    error::Error,
    modules::{self, Module},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PboTarget {
    BySource,
    Release,
    Nowhere,
}

pub struct Executor {
    modules: Vec<Box<dyn Module>>,
    build_pbo: PboTarget,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            build_pbo: PboTarget::Nowhere,
        }
    }

    pub fn build_pbo(&mut self, build_pbo: PboTarget) {
        self.build_pbo = build_pbo;
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }

    pub fn init(&mut self, ctx: &mut Context) -> Result<(), Error> {
        for module in &mut self.modules {
            module.init(ctx)?;
        }
        Ok(())
    }

    pub fn check(&self, ctx: &Context) -> Result<(), Error> {
        for module in &self.modules {
            module.check(ctx)?;
        }
        Ok(())
    }

    pub fn build(&self, ctx: &Context) -> Result<(), Error> {
        for module in &self.modules {
            module.pre_build(ctx)?;
        }
        if self.build_pbo == PboTarget::BySource {
            modules::pbo::dev(ctx)?;
        }
        for module in &self.modules {
            module.post_build(ctx)?;
        }
        Ok(())
    }

    pub fn release(&self, ctx: &Context) -> Result<(), Error> {
        for module in &self.modules {
            module.pre_release(ctx)?;
        }
        if self.build_pbo == PboTarget::Release {
            modules::pbo::release(ctx)?;
        }
        for module in &self.modules {
            module.post_release(ctx)?;
        }
        Ok(())
    }
}
