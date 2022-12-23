use hemtt_bin_error::Error;

use crate::{
    context::Context,
    modules::{self, pbo::Collapse, Module},
};

pub struct Executor<'a> {
    ctx: &'a Context,
    modules: Vec<Box<dyn Module>>,
    collapse: Collapse,
}

impl<'a> Executor<'a> {
    pub fn new(ctx: &'a Context) -> Self {
        Self {
            ctx,
            modules: Vec::new(),
            collapse: Collapse::Yes,
        }
    }

    pub fn collapse(&mut self, collpase: Collapse) {
        self.collapse = collpase;
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }

    pub fn init(&mut self) -> Result<(), Error> {
        for module in &mut self.modules {
            module.init(self.ctx)?;
        }
        Ok(())
    }

    pub fn check(&self) -> Result<(), Error> {
        for module in &self.modules {
            module.check(self.ctx)?;
        }
        Ok(())
    }

    pub fn build(&self) -> Result<(), Error> {
        for module in &self.modules {
            module.pre_build(self.ctx)?;
        }
        modules::pbo::build(self.ctx, self.collapse)?;
        for module in &self.modules {
            module.post_build(self.ctx)?;
        }
        Ok(())
    }

    pub fn release(&self) -> Result<(), Error> {
        for module in &self.modules {
            module.pre_release(self.ctx)?;
        }
        // TODO: Release
        for module in &self.modules {
            module.post_release(self.ctx)?;
        }
        Ok(())
    }
}
