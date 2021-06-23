use std::time::Instant;

use rayon::prelude::*;

mod stage;
pub use stage::Stage;
mod task;
pub use task::Task;

use crate::{
    context::{AddonListContext, Context},
    HEMTTError, Project,
};
use hemtt::Addon;

pub struct Flow {
    pub tasks: Vec<Box<dyn Task>>,
}

impl Flow {
    /// Execute the flow against a vector of addons
    pub fn execute(
        &self,
        addons: Vec<Addon>,
        stages: Vec<Stage>,
        p: &Project,
    ) -> Result<(), HEMTTError> {
        let mut ctx = Context::new(p)?;

        for task in &self.tasks {
            if task.name().len() > ctx.task_pad() {
                ctx.set_task_pad(task.name().len());
            }
        }

        let mut ctx_addons = ctx.get_list(addons)?;

        for stage in stages {
            for task in &self.tasks {
                if ctx_addons.addons().is_empty() {
                    continue;
                }
                if !ctx_addons.failed() && task.hooks().contains(&stage) {
                    ctx_addons
                        .global()
                        .set_message_info(stage.to_string(), task.name());
                    debug!(
                        "[{}] [{:^width$}] Starting",
                        stage,
                        task.name(),
                        width = ctx_addons.global().task_pad()
                    );
                    let start = Instant::now();
                    self.call(&stage, &**task, &mut ctx_addons)?;
                    let elapsed = start.elapsed();
                    info!(
                        "[{}] [{:^width$}] Completed in {} ms",
                        stage,
                        task.name(),
                        elapsed.as_secs_f32() * 1000f32
                            + elapsed.subsec_nanos() as f32 / 1_000_000f32,
                        width = ctx_addons.global().task_pad()
                    );
                }
            }
        }

        for addon in ctx_addons.addons() {
            if let Some(e) = addon.get_failed() {
                error!("{}", e);
            }
        }
        Ok(())
    }

    fn call(
        &self,
        stage: &Stage,
        task: &dyn Task,
        addons: &mut AddonListContext,
    ) -> Result<(), HEMTTError> {
        {
            match stage {
                Stage::Check => task.check_single(addons)?,
                Stage::PreBuild => task.prebuild_single(addons)?,
                Stage::Build => task.build_single(addons)?,
                Stage::PostBuild => task.postbuild_single(addons)?,
                Stage::PreRelease => task.prerelease_single(addons)?,
                Stage::Release => task.release_single(addons)?,
                Stage::PostRelease => task.postrelease_single(addons)?,
            };
        }
        addons.mut_addons().par_iter_mut().for_each(|mut addon| {
            if !addon.failed() {
                match match stage {
                    Stage::Check => task.check(&mut addon),
                    Stage::PreBuild => task.prebuild(&mut addon),
                    Stage::Build => task.build(&mut addon),
                    Stage::PostBuild => task.postbuild(&mut addon),
                    Stage::PreRelease => task.prerelease(&mut addon),
                    Stage::Release => task.release(&mut addon),
                    Stage::PostRelease => task.postrelease(&mut addon),
                } {
                    Ok(_) => {}
                    Err(e) => {
                        addon.set_failed(e);
                    }
                };
            }
        });
        let mut failed = false;
        addons.addons().iter().for_each(|addon| {
            if addon.failed() {
                failed = true;
                error!(
                    "Unable to build `{}`: {:?}",
                    addon.addon().source(),
                    addon.get_failed()
                )
            }
        });
        if failed {
            std::process::exit(1);
        }
        Ok(())
    }
}
