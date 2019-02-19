use walkdir;
use armake2;

use colored::*;

use std::fs;
use std::fs::File;
use std::io::{Error};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error;

pub fn modtime(addon: String) -> Result<SystemTime, std::io::Error> {
    let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
    for entry in walkdir::WalkDir::new(format!("addons/{}", addon)) {
        let metadata = fs::metadata(entry.unwrap().path())?;
        if let Ok(time) = metadata.modified() {
            if time > recent {
                recent = time;
            }
        }
    }
    Ok(recent)
}

pub fn build(p: &crate::project::Project) -> Result<(), std::io::Error> {
    for entry in fs::read_dir("addons")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() { continue };
        let cpath = path.clone().to_str().unwrap().replace(r#"\"#,"/");
        let mut s = cpath.split("/");
        s.next();
        let name = s.next().unwrap().trim();
        let modified = modtime(name.to_owned())?;
        if Path::new(&format!("addons/{}_{}.pbo", p.prefix, name)).exists() {
            let metadata = fs::metadata(format!("addons/{}_{}.pbo", p.prefix, name)).unwrap();
            if let Ok(time) = metadata.modified() {
                if time >= modified {
                    println!("  {} {}", "Skipping".white().bold(), name);
                    continue;
                }
            }
        }
        println!("  {} {}", "Building".green().bold(), name);
        let mut outf = File::create(&format!("addons/{}_{}.pbo", p.prefix, name))?;
        armake2::pbo::cmd_build(
            path,
            &mut outf,
            &vec![],
            &p.exclude,
            &vec![PathBuf::from("./include"), PathBuf::from(".")],
        )?;
    }
    Ok(())
}

pub fn release(p: &crate::project::Project, version: &String) -> Result<(), Error> {
    println!(" {} release v{}", "Preparing".green().bold(), version);
    if Path::new(&format!("releases/{}", version)).exists() {
        return Err(error!("Release already exists, run with --force to clean"));
    }
    build(&p)?;
    if !Path::new(&format!("releases/{}/@{}/addons", version, p.prefix)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/addons", version, p.prefix))?;
    }
    if !Path::new(&format!("releases/{}/@{}/keys", version, p.prefix)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/keys", version, p.prefix))?;
    }
    for file in &p.files {
        fs::copy(file, format!("releases/{}/@{}/{}", version, p.prefix, file))?;
    }
    if !Path::new("keys").exists() {
        fs::create_dir("keys")?;
    }
    if !Path::new(&format!("keys/{}.bikey", p.prefix)).exists() {
        armake2::sign::cmd_keygen(PathBuf::from(&p.prefix))?;
        fs::rename(format!("{}.bikey", p.prefix), format!("keys/{}.bikey", p.prefix))?;
        fs::rename(format!("{}.biprivatekey", p.prefix), format!("keys/{}.biprivatekey", p.prefix))?;
    }
    fs::copy(format!("keys/{}.bikey", p.prefix), format!("releases/{0}/@{1}/keys/{1}.bikey", version, p.prefix))?;
    for entry in fs::read_dir("addons").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let cpath = path.clone();
        let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
        if !path.ends_with(".pbo") && !cpath.contains(p.prefix.as_str()) {
            continue;
        }
        fs::copy(&cpath, format!("releases/{}/@{}/{}", version, p.prefix, cpath))?;
        println!("   {} {}", "Signing".green().bold(), cpath);
        armake2::sign::cmd_sign(
            PathBuf::from(format!("keys/{}.biprivatekey", p.prefix)),
            PathBuf::from(format!("releases/{}/@{}/{}", version, p.prefix, cpath)),
            Some(PathBuf::from(format!("releases/{0}/@{1}/{2}.{0}.bisign", version, p.prefix, cpath))),
            armake2::sign::BISignVersion::V3
        )?;
    }
    Ok(())
}
