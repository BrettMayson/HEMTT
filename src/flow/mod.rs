use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

mod report;
pub use report::Report;

use crate::{HEMTTError, Addon, Project};

#[derive(Clone)]
pub struct Step {
    pub tasks: Vec<Box<dyn Task>>,
    pub name: String,
    pub emoji: String,
}
impl Step {
    pub fn new(emoji: &str, name: &str, tasks: Vec<Box<dyn Task>>) -> Self {
        Self {
            emoji: emoji.to_string(),
            name: name.to_string(),
            tasks,
        }
    }
}

#[derive(Clone)]
pub struct Flow {
    pub steps: Vec<Step>,
}

impl Flow {
    pub fn execute(&self, addons: Vec<Addon>, p: &mut Project) -> Result<(), HEMTTError> {
        let mut addons: Vec<Result<(Report, Addon), HEMTTError>> = addons.into_iter().map(|addon| Ok((Report::new(), addon))).collect();

        for step in &self.steps {
            addons = self.step(&step.emoji, &step.name, &step.tasks, addons, p)?;
        }

        for data in addons {
            match data {
                Ok((report, _)) => {
                    report.display();
                },
                Err(e) => {
                    error!(format!("{}", e));
                }
            }
        }
        Ok(())
    }

    pub fn step(&self, emoji: &str, name: &str, tasks: &Vec<Box<dyn Task>>, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &mut Project) -> Result<Vec<Result<(Report, Addon), HEMTTError>>, HEMTTError>{

        let addon_style = ProgressStyle::default_spinner()
            .tick_chars("\\|/| ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        let master_style = ProgressStyle::default_bar()
            .template("{prefix:.bold.green} {spinner:.yellow} [{elapsed_precise}] [{bar:30.cyan/blue}] [{pos}|{len}]")
            .progress_chars("#>-");

        // Create a multiprogress bar
        let m = MultiProgress::new();
        // Create the top bar
        let total_pb = m.add(ProgressBar::new(addons.len() as u64));
        total_pb.set_style(master_style.clone());
        total_pb.set_prefix(&format!("{} {}", emoji, &fill_space!(" ", 12, name)));

        // Create a progress bar for each addon
        let addons: Vec<Result<(ProgressBar, Report, Addon), HEMTTError>> = addons.into_iter().map(|data| {
            let (report, addon) = data?;
            Ok((m.add(ProgressBar::new(0)), report, addon))
        }).collect();

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
                    },
                    Err(TryRecvError::Disconnected) => { break 'outer; }
                    Err(TryRecvError::Empty) => { break; }
                }
                total_pb.tick();
            }
            total_pb.tick();
        });

        // Task loop
        let addons: Vec<Result<(Report, Addon), HEMTTError>> = addons.into_par_iter().map_with(
                tx.clone(),
                |tx, data: Result<(ProgressBar, Report, Addon), HEMTTError>| -> Result<(Report, Addon), HEMTTError> {
            let (pb, mut report, mut addon) = data?;
            pb.set_style(addon_style.clone());
            pb.set_prefix(&fill_space!(" ", 16, &addon.name));
            
            for task in tasks {
                if report.can_proceed && task.can_run(&mut addon, &report, p)? {
                    pb.tick();
                    report.absorb(match task.run(&addon, &report, p, &pb) {
                        Ok(v) => v,
                        Err(e) => {
                            pb.finish_and_clear();
                            return Err(e);
                        }
                    });
                }
            }

            pb.finish_and_clear();
            //total_pb.inc(1);
            tx.send(1).unwrap();
            Ok((report, addon))
        }).collect();
        tx.send(0).unwrap();
        draw_thread.join().unwrap();

        // let addons = addons.into_iter().filter(|data| {
        //     if let Ok((report, _)) = data {
        //         if !report.can_proceed {
        //             report.display();
        //             false
        //         } else {
        //             true
        //         }
        //     } else { true }
        // }).collect();

        Ok(addons)
    }
}

// A task is an independent item to be ran
pub trait Task: objekt::Clone + std::marker::Send + std::marker::Sync {
    fn can_run(&self, _addon: &Addon, _r: &Report, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn run(&self, _addon: &Addon, _r: &Report, _p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
}
objekt::clone_trait_object!(Task);
