use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

mod report;
mod step;
mod task;

pub use report::Report;
pub use step::Step;
pub use task::Task;

use crate::{Addon, AddonList, HEMTTError, Project};

#[derive(Clone)]
pub struct Flow {
    pub steps: Vec<Step>,
}

impl Flow {
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
                addons = self.parallel(&step.emoji, &step.name, &step.tasks, addons, p)?;
            } else {
                addons = self.single(&step.emoji, &step.name, &step.tasks, addons, p)?;
            }
        }

        for data in &addons {
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
        name: &str,
        tasks: &[Box<dyn Task>],
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
        if !cfg!(windows) {
            total_pb.set_prefix(&format!("{} {}", emoji, &fill_space!(" ", 12, name)));
        } else {
            total_pb.set_prefix(&fill_space!(" ", 12, name).to_string());
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

        // Draw the multiprogress in another thread
        let draw_thread = thread::spawn(move || {
            m.join().unwrap();
        });
        let (tx, rx) = mpsc::channel();

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

        // Task loop
        let addons: Vec<Result<(Report, Addon), HEMTTError>> = addons
            .into_par_iter()
            .map_with(
                tx.clone(),
                |tx, data: Result<(ProgressBar, Report, Addon), HEMTTError>| -> Result<(Report, Addon), HEMTTError> {
                    let (pb, mut report, addon) = data?;
                    pb.set_style(addon_style.clone());
                    pb.set_prefix(&fill_space!(" ", 16, &addon.name));

                    let add = report.stop.is_none();

                    for task in tasks {
                        if report.stop.is_none() && task.can_run(&addon, &report, p)? {
                            pb.tick();
                            report.absorb(match task.parallel(&addon, &report, p, &pb) {
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
                if let Ok((report, addon)) = data {
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
        name: &str,
        tasks: &[Box<dyn Task>],
        addons: Vec<Result<(Report, Addon), HEMTTError>>,
        p: &mut Project,
    ) -> AddonList {
        if !cfg!(windows) {
            println!("{} {}", emoji, &fill_space!(" ", 12, name).bold().cyan());
        } else {
            println!("{}", &fill_space!(" ", 12, name).bold().cyan());
        }

        let mut addons = addons;

        for task in tasks {
            addons = task.single(addons, p)?;
        }

        let addons = addons
            .into_iter()
            .map(|data| {
                if let Ok((report, addon)) = data {
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
