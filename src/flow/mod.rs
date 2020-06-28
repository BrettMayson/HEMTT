use rayon::prelude::*;

mod report;
mod script;
mod stage;
mod step;
mod task;

pub use report::Report;
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
    pub fn execute(&self, addons: Vec<Addon>, p: &mut Project) -> AddonList {
        let mut addons: Vec<Result<(Report, Addon), HEMTTError>> =
            addons.into_iter().map(|addon| Ok((Report::new(), addon))).collect();

        for step in &self.steps {
            if step.none {
                continue;
            }
            if addons.is_empty() {
                continue;
            }
            if step.parallel {
                addons = self.parallel(step, addons, p)?;
            } else {
                addons = self.single(step, addons, p)?;
            }

            // Check for stopped reports
            let mut can_continue = true;
            addons.iter().for_each(|d| {
                if d.is_err() {
                    can_continue = false;
                } else {
                    let (report, addon) = d.as_ref().unwrap();
                    if let Some((fatal, _)) = report.stop {
                        if fatal {
                            can_continue = false;
                            error!("Unable to build `{}`", addon.folder().display().to_string())
                        }
                    }
                }
            });

            if !can_continue {
                std::process::exit(1);
            }
        }

        for data in &mut addons {
            match data {
                Ok((report, _)) => {
                    report.display();
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
        Ok(addons)
    }

    pub fn parallel(&self, step: &Step, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &mut Project) -> AddonList {
        // Create a progress bar for each addon
        let addons: Vec<Result<(Report, Addon), HEMTTError>> = addons
            .into_iter()
            .map(|data| {
                let (report, addon) = data?;
                Ok((report, addon))
            })
            .collect();

        info!("Starting Parallel Step: {}", step.name);

        // Task loop
        let addons: Vec<Result<(Report, Addon), HEMTTError>> = addons
            .into_par_iter()
            .map(
                |data: Result<(Report, Addon), HEMTTError>| -> Result<(Report, Addon), HEMTTError> {
                    let (mut report, addon) = data?;

                    for task in &step.tasks {
                        if report.stop.is_none() && task.can_run(&addon, &report, p, &step.stage)? {
                            report.absorb(match task.parallel(&addon, &report, p, &step.stage) {
                                Ok(v) => v,
                                Err(e) => {
                                    return Err(e);
                                }
                            });
                        }
                    }
                    Ok((report, addon))
                },
            )
            .collect();

        let addons = addons
            .into_iter()
            .map(|data| {
                if let Ok((mut report, addon)) = data {
                    if report.stop.is_some() {
                        report.display();
                    }
                    Ok((report, addon))
                } else {
                    data
                }
            })
            .collect();

        Ok(addons)
    }

    fn single(&self, step: &Step, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &mut Project) -> AddonList {
        if !step.name.is_empty() {
            info!("Starting Step: {}", step.name);
        }

        let mut addons = addons;

        for task in &step.tasks {
            addons = task.single(addons, p, &step.stage)?;
        }

        let addons = addons
            .into_iter()
            .map(|data| {
                if let Ok((mut report, addon)) = data {
                    if report.stop.is_some() {
                        report.display();
                    }
                    Ok((report, addon))
                } else {
                    data
                }
            })
            .collect();

        Ok(addons)
    }
}
