use crate::error::Error;

use crate::report::Report;
use crate::{
    context::Context,
    modules::{self, Module, pbo::Collapse},
};

pub struct Executor {
    ctx: Context,
    modules: Vec<Box<dyn Module>>,
    collapse: Collapse,
    stages: Vec<&'static str>,
}

impl Executor {
    #[must_use]
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            modules: Vec::new(),
            collapse: Collapse::Yes,
            stages: Vec::new(),
        }
    }

    #[must_use]
    pub const fn ctx(&self) -> &Context {
        &self.ctx
    }

    #[must_use]
    pub fn into_ctx(self) -> Context {
        self.ctx
    }

    pub fn collapse(&mut self, collpase: Collapse) {
        self.collapse = collpase;
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }

    /// The exeuctor will run the `init` phases
    pub fn init(&mut self) {
        self.stages.push("init");
    }

    /// The exeuctor will run the `check` phases
    pub fn check(&mut self) {
        self.stages.push("check");
    }

    /// The exeuctor will run the `build` phases
    pub fn build(&mut self, write: bool) {
        self.stages.push("pre_build");
        if write {
            self.stages.push("build");
            self.stages.push("post_build");
        }
    }

    /// The exeuctor will run the `release` phases
    pub fn release(&mut self, archive: bool) {
        self.stages.push("pre_release");
        if archive {
            self.stages.push("archive");
        }
        self.stages.push("post_release");
    }

    /// Execute the `run` phase
    ///
    /// # Errors
    /// [`Error`] depending on the modules
    pub fn run(&mut self) -> Result<Report, Error> {
        self.modules.sort_by(|a, b| {
            if a.name() == "Stringtables" {
                std::cmp::Ordering::Greater
            } else if b.name() == "Stringtables" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        let mut report = Report::new();
        for stage in self.stages.clone() {
            report.merge(match stage {
                "init" => self.run_modules("init")?,
                "check" => self.run_modules("check")?,
                "pre_build" => self.run_modules("pre_build")?,
                "build" => {
                    trace!("phase: build (start)");
                    let report = modules::pbo::build(&self.ctx, self.collapse)?;
                    trace!("phase: build (done)");
                    report
                }
                "post_build" => self.run_modules("post_build")?,
                "pre_release" => self.run_modules("pre_release")?,
                "archive" => {
                    trace!("phase: release (start)");
                    let report = modules::archive::release(&self.ctx)?;
                    trace!("phase: release (done)");
                    report
                }
                "post_release" => self.run_modules("post_release")?,
                _ => unreachable!(),
            });
            if report.failed() {
                break;
            }
        }
        Ok(report)
    }

    fn run_modules(&mut self, phase: &str) -> Result<Report, Error> {
        let mut report = Report::new();
        for module in &mut self.modules {
            trace!("phase: {} ({}) (start)", phase, module.name());
            report.merge(match phase {
                "init" => module.init(&self.ctx)?,
                "check" => module.check(&self.ctx)?,
                "pre_build" => module.pre_build(&self.ctx)?,
                "post_build" => module.post_build(&self.ctx)?,
                "pre_release" => module.pre_release(&self.ctx)?,
                "archive" => module.archive(&self.ctx)?,
                "post_release" => module.post_release(&self.ctx)?,
                _ => unreachable!(),
            });
            if report.failed() {
                trace!("phase: {} ({}) (failed)", phase, module.name());
                break;
            }
            trace!("phase: {} ({}) (done)", phase, module.name());
        }
        Ok(report)
    }
}
