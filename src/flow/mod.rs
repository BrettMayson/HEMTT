use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use colored::*;
use rayon::prelude::*;

#[cfg(not(windows))]
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
#[cfg(windows)]
use indicatif_windows::{MultiProgress, ProgressBar, ProgressStyle};

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
                addons = self.parallel(&step.emoji, &step, addons, p)?;
            } else {
                addons = self.single(&step.emoji, &step, addons, p)?;
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
                            println!();
                            error!(&format!("Unable to build `{}`", addon.folder().display().to_string()))
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
                    error!(format!("{}", e));
                }
            }
        }
        Ok(addons)
    }

    pub fn parallel(
        &self,
        emoji: &str,
        step: &Step,
        addons: Vec<Result<(Report, Addon), HEMTTError>>,
        p: &mut Project,
    ) -> AddonList {
        let addon_style = ProgressStyle::default_spinner()
            .tick_chars("\\|/| ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        let master_style = ProgressStyle::default_bar()
            .template("{prefix:.bold.cyan/blue} {spinner:.yellow} [{elapsed_precise}] [{bar:30.cyan/blue}] [{pos}|{len}]")
            .progress_chars("#>-");

        // Create a multiprogress bar
        let m = MultiProgress::new();
        // Create the top bar
        let mut total = 0;
        let total_pb = m.add(ProgressBar::new(0));
        total_pb.set_style(master_style.clone());

        if !step.name.is_empty() {
            if !cfg!(windows) {
                total_pb.set_prefix(&format!("{} {}", emoji, &fill_space!(" ", 12, &step.name)));
            } else {
                total_pb.set_prefix(&fill_space!(" ", 12, &step.name).to_string());
            }
        }

        // Create a progress bar for each addon
        let addons: Vec<Result<(ProgressBar, Report, Addon), HEMTTError>> = addons
            .into_iter()
            .map(|data| {
                let (report, addon) = data?;
                if report.stop.is_none() {
                    total += 1;
                    total_pb.set_length(total);
                }
                Ok((m.add(ProgressBar::new(0)), report, addon))
            })
            .collect();

        let draw_thread = if !*crate::NOPB {
            thread::spawn(move || {
                m.join().unwrap();
            })
        } else {
            thread::spawn(|| {})
        };

        let (tx, rx) = mpsc::channel();

        if !*crate::NOPB {
            // tick the top bar every 100 ms to keep the multiprogress updated
            thread::spawn(move || 'outer: loop {
                thread::sleep(Duration::from_millis(100));
                loop {
                    match rx.try_recv() {
                        Ok(v) => {
                            if v == 0 {
                                total_pb.finish();
                                break 'outer;
                            } else {
                                total_pb.inc(v);
                            }
                        }
                        Err(TryRecvError::Disconnected) => {
                            break 'outer;
                        }
                        Err(TryRecvError::Empty) => {
                            break;
                        }
                    }
                    total_pb.tick();
                }
                total_pb.tick();
            });
        } else if !cfg!(windows) {
            println!("{} {}", emoji, &fill_space!(" ", 12, &step.name).bold().cyan());
        } else {
            println!("{}", &fill_space!(" ", 12, &step.name).bold().cyan());
        }

        // Task loop
        let addons: Vec<Result<(Report, Addon), HEMTTError>> = addons
            .into_par_iter()
            .map_with(
                tx.clone(),
                |tx, data: Result<(ProgressBar, Report, Addon), HEMTTError>| -> Result<(Report, Addon), HEMTTError> {
                    let (pb, mut report, addon) = data?;

                    if !*crate::NOPB {
                        pb.set_style(addon_style.clone());
                        pb.set_prefix(&fill_space!(" ", 16, &addon.name));
                    }

                    let add = report.stop.is_none();

                    for task in &step.tasks {
                        if report.stop.is_none() && task.can_run(&addon, &report, p, &step.stage)? {
                            pb.tick();
                            report.absorb(match task.parallel(&addon, &report, p, &step.stage, &pb) {
                                Ok(v) => v,
                                Err(e) => {
                                    pb.finish_and_clear();
                                    return Err(e);
                                }
                            });
                        }
                    }

                    pb.finish_and_clear();
                    if add {
                        tx.send(1).unwrap();
                    }
                    Ok((report, addon))
                },
            )
            .collect();

        tx.send(0).unwrap();
        draw_thread.join().unwrap();

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

    fn single(
        &self,
        emoji: &str,
        step: &Step,
        addons: Vec<Result<(Report, Addon), HEMTTError>>,
        p: &mut Project,
    ) -> AddonList {
        if !step.name.is_empty() {
            if !cfg!(windows) {
                println!("{} {}", emoji, &fill_space!(" ", 12, &step.name).bold().cyan());
            } else {
                println!("{}", &fill_space!(" ", 12, &step.name).bold().cyan());
            }
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
