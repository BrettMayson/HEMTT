use std::fs::{create_dir_all, remove_dir_all};

use hemtt_bin_error::Error;
use hemtt_pbo::{prefix::FILES, Prefix};

#[cfg(windows)]
use crate::utils::create_link;

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
        setup_tmp(self.ctx)?;
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
        modules::archive::release(self.ctx)?;
        for module in &self.modules {
            module.post_release(self.ctx)?;
        }
        Ok(())
    }
}

fn setup_tmp(ctx: &Context) -> Result<(), Error> {
    if ctx.tmp().exists() {
        remove_dir_all(ctx.tmp())?;
    }
    create_dir_all(ctx.tmp().join("output"))?;
    let tmp = ctx.tmp().join("source");
    create_dir_all(&tmp)?;
    for addon in ctx.addons() {
        for file in FILES {
            let root = ctx.vfs().join(addon.folder()).unwrap();
            let path = root.join(file).unwrap();
            if path.exists().unwrap() {
                let prefix = Prefix::new(
                    &path.read_to_string().unwrap(),
                    ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                )?
                .into_inner();
                let tmp_addon = tmp.join(prefix);
                create_dir_all(tmp_addon.parent().unwrap())?;
                let target = std::env::current_dir()?
                    .join(root.as_str().trim_start_matches('/').replace('/', "\\"));
                create_link(tmp_addon.to_str().unwrap(), target.to_str().unwrap())?;
                break;
            }
        }
    }
    let include = std::env::current_dir().unwrap().join("include");
    if !include.exists() {
        return Ok(());
    }
    for outer_prefix in std::fs::read_dir(include)? {
        let outer_prefix = outer_prefix?.path();
        if outer_prefix.is_dir() {
            let tmp_outer_prefix = tmp.join(outer_prefix.file_name().unwrap());
            for prefix in std::fs::read_dir(outer_prefix)? {
                let prefix = prefix?.path();
                if prefix.is_dir() {
                    let tmp_mod = tmp_outer_prefix.join(prefix.file_name().unwrap());
                    create_dir_all(tmp_mod.parent().unwrap())?;
                    create_link(tmp_mod.to_str().unwrap(), prefix.to_str().unwrap())?;
                }
            }
        }
    }
    Ok(())
}
