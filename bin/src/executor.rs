use std::fs::{create_dir_all, remove_dir_all};

use crate::error::Error;
use crate::link::create_link;

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
    #[must_use]
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

    /// Initialize the modules
    ///
    /// # Errors
    /// [`Error::Workspace`] if the workspace could not be initialized
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

    /// Execute the `check` phase
    ///
    /// # Errors
    /// Any error that occurs during the `check` phase
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

    /// Execute the `build` phase
    ///
    /// # Errors
    /// Any error that occurs during the `pre_build`, `build` or `post_build` phase
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

    /// Execute the `release` phase
    ///
    /// # Errors
    /// Any error that occurs during the `pre_release`, `release` or `post_release` phase
    pub fn release(&self, archive: bool) -> Result<(), Error> {
        trace!("phase: pre_release (start)");
        for module in &self.modules {
            trace!("phase: pre_release ({}) (start)", module.name());
            module.pre_release(self.ctx)?;
            trace!("phase: pre_release ({}) (done)", module.name());
        }
        trace!("phase: pre_release (done)");
        trace!("phase: release (start)");
        if archive {
            modules::archive::release(self.ctx)?;
        }
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
        let tmp_addon = tmp.join(addon.prefix().as_pathbuf());
        create_dir_all(tmp_addon.parent().unwrap())?;
        let target = ctx.project_folder().join(
            addon
                .folder()
                .as_str()
                .trim_start_matches('/')
                .replace('/', "\\"),
        );
        create_link(&tmp_addon, &target)?;
    }
    // maybe replace with config or rhai in the future?
    let addons = ctx.project_folder().join("addons");
    for file in std::fs::read_dir(addons)? {
        let file = file?.path();
        if file.is_dir() {
            continue;
        }
        let tmp_file = tmp
            .join(ctx.addons().first().unwrap().prefix().as_pathbuf())
            .parent()
            .unwrap()
            .join(file.file_name().unwrap());
        if file.metadata()?.len() > 1024 * 1024 * 10 {
            warn!(
                "File `{}` is larger than 10MB, this will slow builds.",
                file.display()
            );
        }
        trace!("copying `{}` to tmp for binarization", file.display());
        std::fs::copy(&file, &tmp_file)?;
    }
    let include = ctx.project_folder().join("include");
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
                    create_link(&tmp_mod, &prefix)?;
                }
            }
        }
    }
    Ok(())
}
