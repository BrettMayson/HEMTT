use rayon::prelude::*;

mod script;
mod stage;
mod step;
mod task;

pub use script::BuildScript;
pub use script::Script;
pub use stage::Stage;
pub use step::Step;
pub use task::Task;

use crate::{Addon, AddonList, HEMTTError, Project};

#[derive(Clone)]
pub struct Flow {
    pub steps: Vec<Step>,
}

impl Flow {
    /// Execute the flow against a vector of addons
    pub fn execute(&self, addons: Vec<Addon>, p: &mut Project) -> Result<AddonList, HEMTTError> {
        let mut addons: AddonList = addons.into_iter().map(|addon| Ok((true, false, addon))).collect();

        for step in &self.steps {
            if step.none {
                continue;
            }
            if addons.is_empty() {
                continue;
            }
            if addons.iter().any(|ra| if let Ok(a) = ra { !a.1 } else { false }) {
                if step.parallel {
                    addons = self.parallel(step, addons, p)?;
                } else {
                    addons = self.single(step, addons, p)?;
                }
            }

            // Check for stopped reports
            let mut can_continue = true;
            addons.iter().for_each(|d| {
                if d.is_err() {
                    can_continue = false;
                } else {
                    let (ok, _, addon) = d.as_ref().unwrap();
                    if !ok {
                        can_continue = false;
                        error!("Unable to build `{}`", addon.folder().display().to_string())
                    }
                }
            });

            if !can_continue {
                std::process::exit(1);
            }
        }

        for data in &mut addons {
            if let Err(e) = data {
                error!("{}", e);
            }
        }
        Ok(addons)
    }

    pub fn parallel(&self, step: &Step, addons: AddonList, p: &mut Project) -> Result<AddonList, HEMTTError> {
        info!("Starting Parallel Step: {}", step.name);

        // Task loop
        let addons: AddonList = addons
            .into_par_iter()
            .map(
                |data: Result<(bool, bool, Addon), HEMTTError>| -> Result<(bool, bool, Addon), HEMTTError> {
                    let (mut ok, mut skip, addon) = data?;
                    for task in &step.tasks {
                        if ok && !skip && task.can_run(&addon, p, &step.stage)? {
                            trace!("[{}] running task: {}", addon.name, step.stage);
                            match task.parallel(&addon, p, &step.stage) {
                                Ok(v) => {
                                    ok = ok && v.0;
                                    skip = skip || v.1;
                                    trace!("[{}] skipping future tasks", addon.name);
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            };
                        }
                    }
                    Ok((ok, skip, addon))
                },
            )
            .collect();

        Ok(addons)
    }

    fn single(&self, step: &Step, addons: AddonList, p: &mut Project) -> Result<AddonList, HEMTTError> {
        if !step.name.is_empty() {
            info!("Starting Step: {}", step.name);
        }

        let mut addons = addons;

        for task in &step.tasks {
            addons = task.single(addons, p, &step.stage)?;
        }

        Ok(addons)
    }
}
