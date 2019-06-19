use colored::*;
use glob::glob;
use rayon::prelude::*;
use reqwest;

use std::fs;
use std::fs::File;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::project;
use crate::error::*;

pub fn clear_pbos(p: &project::Project, addons: &[PathBuf]) -> Result<(), Error> {
    let count = Arc::new(Mutex::new(0));
    addons.par_iter()
        .for_each(|folder| {
            let mut target = folder.parent().unwrap().to_path_buf();
            if p.prefix.is_empty() {
                target.push(&format!("{}.pbo", folder.file_name().unwrap().to_str().unwrap()));
            } else {
                target.push(&format!("{}_{}.pbo", p.prefix, folder.file_name().unwrap().to_str().unwrap()));
            }
            if target.exists() {
                let mut data = count.lock().unwrap();
                *data += 1;
                fs::remove_file(target).print();
            }
        });
    yellow!("Cleaned", format!("{} PBOs", *count.lock().unwrap()));
    Ok(())
}

pub fn clear_pbo(p: &project::Project, source: &PathBuf) -> Result<(), Error> {
    let mut target = source.parent().unwrap().to_path_buf();
    let name = source.file_name().unwrap().to_str().unwrap().to_owned();
    target.push(&format!("{}_{}.pbo", p.prefix, name));
    if target.exists() {
        fs::remove_file(target)?;
    }
    Ok(())
}

pub fn clear_release(p: &project::Project, version: &str) -> Result<(), Error> {
    if Path::new(&format!("releases/{}", version)).exists() {
        println!("  {} old release v{}", "Cleaning".yellow().bold(), version);
        fs::remove_dir_all(format!("releases/{}", version))?;
    }

    // Keys
    let keyname = p.get_keyname();
    let keypath = &format!("releases/keys/{}.bikey", keyname);
    let pkeypath = &format!("releases/keys/{}.biprivatekey", keyname);

    if Path::new(keypath).exists() {
        println!("  {} old key {}", "Cleaning".yellow().bold(), keyname);
        fs::remove_file(keypath)?;

        if !p.reuse_private_key {
            if Path::new(pkeypath).exists() {
                fs::remove_file(pkeypath)?;
            }
        }
    }

    Ok(())
}

pub fn clear_releases(p: &project::Project) -> Result<(), Error> {
    println!("  {} all releases", "Cleaning".yellow().bold());
    if Path::new("releases").exists() {
        if !p.reuse_private_key {
            fs::remove_dir_all("releases")?;
        } else {
            for entry in glob("releases/*.*.*").unwrap_or_print() {
                if let Ok(path) = entry {
                    fs::remove_dir_all(path)?;
                }
            }
            for entry in glob("releases/keys/*.bikey").unwrap_or_print() {
                if let Ok(path) = entry {
                    fs::remove_file(path)?;
                }
            }
        }
    }
    Ok(())
}
