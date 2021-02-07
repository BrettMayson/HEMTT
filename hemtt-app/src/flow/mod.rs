use std::time::Instant;

use rayon::prelude::*;

mod stage;
pub use stage::Stage;
mod task;
pub use task::Task;

use crate::{context::Context, AddonList, HEMTTError, Project};
use hemtt::Addon;

// #[derive(Clone)]
pub struct Flow {
    pub tasks: Vec<Box<dyn Task>>,
}

impl Flow {
    /// Execute the flow against a vector of addons
    pub fn execute(&self, addons: Vec<Addon>, p: &Project) -> Result<AddonList, HEMTTError> {
        let mut addons: AddonList = addons
            .into_iter()
            .map(|addon| Ok((true, false, addon)))
            .collect();

        let mut ctx = Context::new(p)?;

        for task in &self.tasks {
            if task.name().len() > ctx.task_pad {
                ctx.task_pad = task.name().len();
            }
        }

        for stage in Stage::all() {
            for task in &self.tasks {
                if addons.is_empty() {
                    continue;
                }
                if addons
                    .iter()
                    .any(|ra| if let Ok(a) = ra { !a.1 } else { false })
                    && task.hooks().contains(&stage)
                {
                    info!(
                        "[{}] [{:^width$}] Starting",
                        stage,
                        task.name(),
                        width = ctx.task_pad
                    );
                    let start = Instant::now();
                    addons = self.call(&stage, &**task, addons, &mut ctx)?;
                    let elapsed = start.elapsed();
                    info!(
                        "[{}] [{:^width$}] Completed in {} ms",
                        stage,
                        task.name(),
                        elapsed.as_secs_f32() * 1000f32
                            + elapsed.subsec_nanos() as f32 / 1_000_000f32,
                        width = ctx.task_pad
                    );
                }
            }
        }

        for data in &mut addons {
            if let Err(e) = data {
                error!("{}", e);
            }
        }
        Ok(addons)
    }

    fn call(
        &self,
        stage: &Stage,
        task: &dyn Task,
        addons: AddonList,
        ctx: &mut Context,
    ) -> Result<AddonList, HEMTTError> {
        let mut addons = addons;
        {
            let mut actx = ctx.get_list(&mut addons);
            match stage {
                Stage::Check => task.check_single(&mut actx)?,
                Stage::PreBuild => task.prebuild_single(&mut actx)?,
                Stage::Build => task.build_single(&mut actx)?,
                Stage::PostBuild => task.postbuild_single(&mut actx)?,
                Stage::Release => task.release_single(&mut actx)?,
                Stage::PostRelease => task.postrelease_single(&mut actx)?,
                Stage::Script => {}
                Stage::None => {}
            };
        }
        self.can_continue(&addons);
        addons = {
            let addons: AddonList = addons
            .into_par_iter()
            .map(
                |data: Result<(bool, bool, Addon), HEMTTError>| -> Result<(bool, bool, Addon), HEMTTError> {
                    let (mut ok, mut skip, addon) = data?;
                    if ok && !skip {
                        let mut actx = ctx.get_single(&addon);
                        match match stage {
                            Stage::Check => task.check(&mut actx),
                            Stage::PreBuild => task.prebuild(&mut actx),
                            Stage::Build => task.build(&mut actx),
                            Stage::PostBuild => task.postbuild(&mut actx),
                            Stage::Release => task.release(&mut actx),
                            Stage::PostRelease => task.postrelease(&mut actx),
                            Stage::Script => {Ok((true, false))}
                            Stage::None => {Ok((true, false))}
                        } {
                            Ok(v) => {
                                ok = ok && v.0;
                                skip = skip || v.1;
                                if !ok || skip {
                                    trace!("[{}] skipping future tasks", addon.name());
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        };
                    }
                    Ok((ok, skip, addon))
                },
            )
            .collect();
            addons
        };
        Ok(addons)
    }

    fn can_continue(&self, addons: &AddonList) {
        let mut can_continue = true;
        addons.iter().for_each(|d| {
            match d {
                Ok((ok, _, addon)) => {
                    if !ok {
                        can_continue = false;
                        error!("Unable to build `{}`", addon.source())
                    }
                }
                Err(e) => {
                    error!("{}", e);
                    can_continue = false;
                }
            }
        });

        if !can_continue {
            std::process::exit(1);
        }
    }
}
