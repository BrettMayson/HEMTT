use walkdir;
use armake2;

use colored::*;

use std::fs;
use std::fs::{File, DirEntry};
use std::io::{Error};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error;
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

pub fn build(p: &crate::project::Project) -> Result<(), Error> {
    for entry in fs::read_dir("addons")? {
        let entry = entry?;
        if !entry.path().is_dir() { continue }
        let name = entry.file_name().into_string().unwrap();
        if p.skip.contains(&name) { continue }
        let target = PathBuf::from(&format!("addons/{}_{}.pbo", p.prefix, &name));
        _build(&p, &entry.path(), &target, &name)?;
    }
    for opt in &p.optionals {
        if p.skip.contains(opt) { continue }
        let source = PathBuf::from(&format!("optionals/{}", opt));
        let target = PathBuf::from(&format!("optionals/{}_{}.pbo", p.prefix, opt));
        _build(&p, &source, &target, &opt)?;
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
    if !Path::new("releases/keys").exists() {
        fs::create_dir("releases/keys")?;
    }
    if !Path::new(&format!("releases/keys/{}.bikey", p.prefix)).exists() {
        println!(" {} {}.bikey", "Generating".green().bold(), p.prefix);
        armake2::sign::cmd_keygen(PathBuf::from(&p.prefix))?;
        fs::rename(format!("{}.bikey", p.prefix), format!("releases/keys/{}.bikey", p.prefix))?;
        fs::rename(format!("{}.biprivatekey", p.prefix), format!("releases/keys/{}.biprivatekey", p.prefix))?;
    }
    fs::copy(format!("releases/keys/{}.bikey", p.prefix), format!("releases/{0}/@{1}/keys/{1}.bikey", version, p.prefix))?;
    for entry in fs::read_dir("addons").unwrap() {
        _copy_sign(&entry.unwrap(), &p, &version)?;
    }
    if Path::new("optionals").exists() {
        if !Path::new(&format!("releases/{}/@{}/optionals", version, p.prefix)).exists() {
            fs::create_dir_all(format!("releases/{}/@{}/optionals", version, p.prefix))?;
        }
        for entry in fs::read_dir("optionals").unwrap() {
            _copy_sign(&entry.unwrap(), &p, &version)?;
        }
    }
    Ok(())
}

fn _build(p: &crate::project::Project, source: &Path, target: &Path, name: &str) -> Result<bool, Error> {
    let modified = modtime(source)?;
    if target.exists() {
        let metadata = fs::metadata(target).unwrap();
        if let Ok(time) = metadata.modified() {
            if time >= modified {
                println!("  {} {}", "Skipping".white().bold(), name);
                return Ok(false);
            }
        }
    }

    println!("  {} {}", "Building".green().bold(), name);
    let mut outf = File::create(target)?;

    let mut include = p.include.to_owned();
    include.push(PathBuf::from("."));

    armake2::pbo::cmd_build(
        source.to_path_buf(),
        &mut outf,
        &vec![],
        &p.exclude,
        &include,
    ).print_error(false);
    Ok(true)
}

fn _copy_sign(entry: &DirEntry, p: &crate::project::Project, version: &String) -> Result<bool, Error> {
    let path = entry.path();
    let cpath = path.clone();
    let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
    if !path.ends_with(".pbo") && !cpath.contains(p.prefix.as_str()) {
        return Ok(false);
    }
    fs::copy(&cpath, format!("releases/{}/@{}/{}", version, p.prefix, cpath))?;
    println!("   {} {}", "Signing".green().bold(), cpath);
    armake2::sign::cmd_sign(
        PathBuf::from(format!("releases/keys/{}.biprivatekey", p.prefix)),
        PathBuf::from(format!("releases/{}/@{}/{}", version, p.prefix, cpath)),
        Some(PathBuf::from(format!("releases/{0}/@{1}/{2}.{0}.bisign", version, p.prefix, cpath))),
        armake2::sign::BISignVersion::V3
    )?;
    Ok(true)
}
