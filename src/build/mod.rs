pub mod release;
pub mod script;
pub mod sign;
mod result;

use colored::*;
use pbr::ProgressBar;
use rayon::prelude::*;

use std::fs;
use std::fs::File;
use std::io::{Error};
use std::iter::repeat;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, Instant};

pub use crate::build::result::{BuildResult, PBOResult};
use crate::error::*;

pub fn modtime(addon: &Path) -> Result<SystemTime, Error> {
    let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
    for entry in walkdir::WalkDir::new(addon) {
        let metadata = fs::metadata(entry.unwrap().path())?;
        if let Ok(time) = metadata.modified() {
            if time > recent {
                recent = time;
            }
        }
    }
    Ok(recent)
}

pub fn addons(p: &crate::project::Project, addons: &Vec<PathBuf>) -> Result<BuildResult, Error> {
    println!("  {} {}", "Building".green().bold(), addons.len());
    let mut pb = ProgressBar::new(addons.len() as u64);
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.set_width(Some(60));
    let pbm = Arc::new(Mutex::new(pb));
    let mut buildresult = BuildResult::new();
    let result = Arc::new(Mutex::new(&mut buildresult));
    addons.par_iter().for_each(|entry| {
        let name = entry.file_name().unwrap().to_str().unwrap().to_owned();
        let mut target = entry.parent().unwrap().to_path_buf();
        target.push(&format!("{}_{}.pbo", p.prefix, &name));
        let now = Instant::now();
        let buildresult = _build(&p, &entry, &target, Some(&pbm)).unwrap_or_print();
        let elapsed = now.elapsed();
        let pboresult = PBOResult::new(
            entry.clone(),
            target.clone(),
            ((elapsed.as_secs() as u128) * 1000) + (elapsed.subsec_millis() as u128),
        );
        match buildresult {
            0 => result.lock().unwrap().skipped.push(pboresult),
            1 => result.lock().unwrap().built.push(pboresult),
            2 => result.lock().unwrap().failed.push(pboresult),
            _ => (),
        }
        pbm.lock().unwrap().inc();
        print!("\r");
        eprint!("\r");
    });
    pbm.lock().unwrap().finish_print(&format!("\r     {} {} {}", match buildresult.failed.len() {
        0 => "Built".green().bold(),
        _ => "Built".yellow().bold()
    }, buildresult.built.len(), crate::repeat!(" ", 50)));
    println!();
    if buildresult.failed.len() != 0 {
        println!("    {} {} {:?}", "Failed".red().bold(), buildresult.failed.len(), buildresult.failed);
    }
    if buildresult.skipped.len() != 0 {
        println!("   {} {}", "Skipped".bold(), buildresult.skipped.len());
    }
    Ok(buildresult)
}

fn _build(p: &crate::project::Project, source: &PathBuf, target: &PathBuf, pbm: Option<&Arc<Mutex<ProgressBar<std::io::Stdout>>>>) -> Result<u32, Error> {
    let modified = modtime(source)?;
    if target.exists() {
        let metadata = fs::metadata(target).unwrap();
        if let Ok(time) = metadata.modified() {
            if time >= modified {
                // println!("\r  {} {}{}", "Skipping".white().bold(), name, crate::repeat!(" ", 50 - name.len()));
                if let Some(ref pb) = pbm {
                    let mut pbu = pb.lock().unwrap();
                    pbu.tick();
                    print!("\r");
                    eprint!("\r");
                }
                return Ok(0);
            }
        }
    }

    // println!("\r  {} {}{}", "Building".green().bold(), name, crate::repeat!(" ", 50 - name.len()));
    if let Some(ref pb) = pbm {
        let mut pbu = pb.lock().unwrap();
        pbu.tick();
        print!("\r");
        eprint!("\r");
    }
    let mut outf = File::create(target)?;

    let mut include = p.include.to_owned();
    include.push(PathBuf::from("."));

    if let Err(ref error) = armake2::pbo::cmd_build(
        source.to_path_buf(),   // Source
        &mut outf,              // Target
        &p.get_headerexts(),    // Header extensions
        &p.exclude,             // Exclude files glob patterns
        &include,               // Include folders
    ) {
        let name = source.to_str().unwrap().to_owned();
        eprintln!("\r{}: {}{}\n{}", "error".red().bold(), name, crate::repeat!(" ", 53 - name.len()), error);
        if target.exists() {
            fs::remove_file(target)?;
        }
        return Ok(2);
    }
    Ok(1)
}
