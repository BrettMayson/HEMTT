pub mod release;
pub mod sign;

use colored::*;
use pbr::ProgressBar;
use rayon::prelude::*;

use std::fs;
use std::fs::File;
use std::io::{Error};
use std::iter::repeat;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

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

pub fn many(p: &crate::project::Project, addons: Vec<PathBuf>) -> Result<bool, Error> {
    println!("  {} {}", "Building".green().bold(), addons.len());
    let mut pb = ProgressBar::new(addons.len() as u64);
    pb.format("╢▌▌░╟");
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.set_width(Some(60));
    let pbm = Arc::new(Mutex::new(pb));
    let skip = Arc::new(Mutex::new(0));
    let built = Arc::new(Mutex::new(0));
    let err = Arc::new(Mutex::new(0));
    addons.par_iter().for_each(|entry| {
        let name = entry.file_name().unwrap().to_str().unwrap().to_owned();
        let mut target = entry.parent().unwrap().to_path_buf();
        target.push(&format!("{}_{}.pbo", p.prefix, &name));
        match _build(&p, &entry, &target, Some(&pbm)).unwrap() {
            0 => *skip.lock().unwrap() += 1,
            1 => *built.lock().unwrap() += 1,
            2 => *err.lock().unwrap() += 1,
            _ => (),
        }
        pbm.lock().unwrap().inc();
        print!("\r");
        eprint!("\r");
    });
    let built = *built.lock().unwrap();
    let errors = *err.lock().unwrap();
    pbm.lock().unwrap().finish_print(&format!("\r     {} {} {}", match errors {
        0 => "Built".green().bold(),
        _ => "Built".yellow().bold()
    }, built, crate::repeat!(" ", 50)));
    println!();
    if errors != 0 {
        println!("    {} {}", "Failed".red().bold(), errors);
    }
    let skip = *skip.lock().unwrap();
    if skip != 0 {
        println!("   {} {}", "Skipped".bold(), skip);
    }
    Ok(errors == 0)
}

pub fn single(p: &crate::project::Project, source: &PathBuf) -> Result<(), Error> {
    let name = source.file_name().unwrap().to_str().unwrap().to_owned();
    let mut target = source.parent().unwrap().to_path_buf();
    target.push(&format!("{}_{}.pbo", p.prefix, &name));
    _build(&p, &source, &target, None).unwrap();
    Ok(())
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
