use std::fs::{create_dir_all, remove_dir_all};

use hemtt_bin_error::Error;
use hemtt_pbo::{prefix::FILES, Prefix};

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
        trace!("phase: init (start)");
        setup_tmp(self.ctx)?;
        for module in &mut self.modules {
            trace!("phase: init ({}) (start)", module.name());
            module.init(self.ctx)?;
            trace!("phase: init ({}) (done)", module.name());
        }
        trace!("phase: init (done)");
        Ok(())
    }

    pub fn check(&self) -> Result<(), Error> {
        trace!("phase: check (start)");
        for module in &self.modules {
            trace!("phase: check ({}) (start)", module.name());
            module.check(self.ctx)?;
            trace!("phase: check ({}) (done)", module.name());
        }
        trace!("phase: check (done)");
        Ok(())
    }

    pub fn build(&self) -> Result<(), Error> {
        trace!("phase: pre_build (start)");
        for module in &self.modules {
            trace!("phase: pre_build ({}) (start)", module.name());
            module.pre_build(self.ctx)?;
            trace!("phase: pre_build ({}) (done)", module.name());
        }
        trace!("phase: pre_build (done)");
        trace!("phase: build (start)");
        modules::pbo::build(self.ctx, self.collapse)?;
        trace!("phase: build (done)");
        trace!("phase: post_build (start)");
        for module in &self.modules {
            trace!("phase: post_build ({}) (start)", module.name());
            module.post_build(self.ctx)?;
            trace!("phase: post_build ({}) (done)", module.name());
        }
        trace!("phase: post_build (done)");
        Ok(())
    }

    pub fn release(&self) -> Result<(), Error> {
        trace!("phase: pre_release (start)");
        for module in &self.modules {
            trace!("phase: pre_release ({}) (start)", module.name());
            module.pre_release(self.ctx)?;
            trace!("phase: pre_release ({}) (done)", module.name());
        }
        trace!("phase: pre_release (done)");
        trace!("phase: release (start)");
        modules::archive::release(self.ctx)?;
        trace!("phase: release (done)");
        trace!("phase: post_release (start)");
        for module in &self.modules {
            trace!("phase: post_release ({}) (start)", module.name());
            module.post_release(self.ctx)?;
            trace!("phase: post_release ({}) (done)", module.name());
        }
        trace!("phase: post_release (done)");
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
