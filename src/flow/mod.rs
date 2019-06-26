use std::time::{Duration, Instant};
use std::thread;
use std::fs::File;
use std::path::PathBuf;

use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use rayon::prelude::*;

mod report;
pub use report::Report;

use crate::{HEMTTError, Addon, Project};

#[derive(Clone)]
pub struct FlowArgs {
    pub flow: Flow,
    pub project: Project,
}

// A flow is a queue of tasks to run a various points during the app cycle
#[derive(Clone)]
pub struct Flow {
    pub checks: Vec<Box<dyn Task>>,
    pub pre_build: Vec<Box<dyn Task>>,
    pub post_build: Vec<Box<dyn Task>>,
    pub release: Vec<Box<dyn Task>>,
}

impl Flow {
    pub fn execute(&self, addons: &[Addon], p: &mut Project) -> Result<(), HEMTTError> {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} [{pos}|{len}] {spinner} {wide_msg}");

        let m = MultiProgress::new();

        let total_pb = m.add(ProgressBar::new(addons.len() as u64));
        total_pb.println("Pre Building");

        let addons: Vec<(ProgressBar, &Addon)> = addons.into_iter().map(|addon| {
            (m.add(ProgressBar::new(
                (self.checks.len() * self.pre_build.len()) as u64
            )), addon)
        }).collect();

        // Progress bar draw thread
        let draw_thread = thread::spawn(move || {
            m.join().unwrap();
        });

        let addons: Vec<Result<(Report, &Addon), HEMTTError>> = addons.into_par_iter().map(
                |data: (ProgressBar, &Addon)| -> Result<(Report, &Addon), HEMTTError> {
            let (pb, mut addon) = data;
            pb.set_style(spinner_style.clone());
            pb.set_prefix(&fill_space!(" ", 16, &addon.name));
            pb.set_message("Checking");
            let mut report = Report::new();
            for task in &self.checks {
                if report.can_proceed && task.chk_can_run(&mut addon, p)? {
                    pb.tick();
                    report.absorb(match task.chk_run(&addon, p, &pb) {
                        Ok(v) => v,
                        Err(e) => {
                            pb.finish_and_clear();
                            return Err(e);
                        }
                    });
                }
                pb.inc(1);
            }
            pb.set_message("Pre Build");
            for task in &self.pre_build {
                if report.can_proceed && task.pre_can_run(&mut addon, p)? {
                    pb.tick();
                    report.absorb(match task.pre_run(&addon, p, &pb) {
                        Ok(v) => v,
                        Err(e) => {
                            total_pb.println(format!("Failed {}", addon.name));
                            pb.finish_and_clear();
                            return Err(e);
                        }
                    });
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
            total_pb.inc(1);
            Ok((report, addon))
        }).collect();
        total_pb.finish_with_message("Prebuild Done");
        draw_thread.join().unwrap();

        let m = MultiProgress::new();
        let total_pb = m.add(ProgressBar::new(addons.len() as u64));
        let addons: Vec<(ProgressBar, Report, &Addon)> = addons.into_iter().map(|data| {
            let (report, addon) = data.unwrap();
            (m.add(ProgressBar::new(0)), report, addon)
        }).collect();

        // Progress bar draw thread
        let draw_thread = thread::spawn(move || {
            m.join();
        });
        total_pb.println("Building");
        let addons: Vec<Result<(Report, &Addon), HEMTTError>> = addons.into_par_iter().map(
                |data: (ProgressBar, Report, &Addon)| -> Result<(Report, &Addon), HEMTTError> {
            let (pb, mut report, addon) = data;
            if report.can_proceed {
                pb.set_style(spinner_style.clone());
                pb.set_prefix(&fill_space!(" ", 16, &addon.name));
                pb.set_message("Building");
                match crate::build::dir(&addon, &pb) {
                    Ok((pbo, rep)) => {
                        report.absorb(rep);
                        let mut target = PathBuf::from(crate::build::folder_name(&addon.location));
                        target.push(&format!("{}_{}.pbo", p.prefix, &addon.name));
                        let mut outf = File::create(target)?;
                        pbo.write(&mut outf)?;
                    },
                    Err(e) => {
                        total_pb.println(format!("Failed {}", addon.name));
                        pb.finish_and_clear();
                        return Err(e);
                    }
                }
                pb.set_message("Waiting");
                total_pb.inc(1);
                pb.finish_and_clear();
            }
            Ok((report, addon))
        }).collect();
        total_pb.finish_with_message("Prebuild Done");
        draw_thread.join().unwrap();
        for data in addons {
            let (report, _) = data?;
            //report.display();
        }
        Ok(())
    }
}

// A task is an independent item to be ran
pub trait Task: objekt::Clone + std::marker::Send + std::marker::Sync {
    fn chk_can_run(&self, _addon: &Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn chk_run(&self, _addon: &Addon, _p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
    fn pre_can_run(&self, _addon: &Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn pre_run(&self, _addon: &Addon, _p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
    fn post_can_run(&self, _addon: &Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn post_run(&self, _addon: &Addon, _p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
    fn rel_can_run(&self, _addon: &Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn rel_run(&self, _addon: &Addon, _p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
}
objekt::clone_trait_object!(Task);
