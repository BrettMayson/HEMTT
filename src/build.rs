use armake2;
use colored::*;
use rayon::prelude::*;
use walkdir;

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
    let dirs: Vec<_> = fs::read_dir("addons").unwrap()
        .map(|file| file.unwrap())
        .filter(|file_or_dir| file_or_dir.path().is_dir())
        .collect();
    dirs.par_iter().for_each(|entry| {
        let name = entry.file_name().into_string().unwrap();
        if p.skip.contains(&name) { return };
        let target = PathBuf::from(&format!("addons/{}_{}.pbo", p.prefix, &name));
        _build(&p, &entry.path(), &target, &name).unwrap();
    });
    &p.optionals.par_iter().for_each(|opt| {
        if p.skip.contains(opt) {return};
        let source = PathBuf::from(&format!("optionals/{}", opt));
        let target = PathBuf::from(&format!("optionals/{}_{}.pbo", p.prefix, opt));
        _build(&p, &source, &target, &opt).unwrap();
    });
    Ok(())
}

pub fn build_single(p: &crate::project::Project, addon: &String) -> Result<(), Error> {
    let source = PathBuf::from(&format!("addons/{}", &addon));
    let target = PathBuf::from(&format!("addons/{}_{}.pbo", p.prefix, &addon));
    _build(&p, &source, &target, &addon)?;
    Ok(())
}

pub fn release(p: &crate::project::Project, version: &String) -> Result<(), Error> {
    // Build
    println!(" {} release v{}", "Preparing".green().bold(), version);
    if Path::new(&format!("releases/{}", version)).exists() {
        return Err(error!("Release already exists, run with --force to clean"));
    }
    build(&p)?;


    let modname = p.get_modname();
    if !Path::new(&format!("releases/{}/@{}/addons", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/addons", version, modname))?;
    }
    if !Path::new(&format!("releases/{}/@{}/keys", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/keys", version, modname))?;
    }
    for file in &p.files {
        fs::copy(file, format!("releases/{}/@{}/{}", version, modname, file))?;
    }

    // Generate key
    if !Path::new("releases/keys").exists() {
        fs::create_dir("releases/keys")?;
    }
    let keyname = p.get_keyname();
    if !Path::new(&format!("releases/keys/{}.bikey", keyname)).exists() {
        println!(" {} {}.bikey", "Generating".green().bold(), keyname);
        armake2::sign::cmd_keygen(PathBuf::from(&keyname))?;
        fs::rename(format!("{}.bikey", keyname), format!("releases/keys/{}.bikey", keyname))?;
        fs::rename(format!("{}.biprivatekey", keyname), format!("releases/keys/{}.biprivatekey", keyname))?;
    }

    // Sign
    fs::copy(format!("releases/keys/{}.bikey", keyname), format!("releases/{}/@{}/keys/{}.bikey", version, modname, keyname))?;

    let mut folder = String::from("addons");
    let dirs: Vec<_> = fs::read_dir(&folder).unwrap()
        .map(|file| file.unwrap())
        .collect();
    dirs.par_iter().for_each(|entry| {
        _copy_sign(&folder, &entry, &p, &version).unwrap();
    });

    folder = String::from("optionals");
    if Path::new(&folder).exists() {
        if !Path::new(&format!("releases/{}/@{}/{}", version, modname, folder)).exists() {
            fs::create_dir_all(format!("releases/{}/@{}/{}", version, modname, folder))?;
        }
        let opts: Vec<_> = fs::read_dir(&folder).unwrap()
            .map(|file| file.unwrap())
            .collect();
        opts.par_iter().for_each(|entry| {
            _copy_sign(&folder, &entry, &p, &version).unwrap();
        });
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
        source.to_path_buf(),   // Source
        &mut outf,              // Target
        &p.get_headerexts(),    // Header extensions
        &p.exclude,             // Exclude files glob patterns
        &include,               // Include folders
    ).print_error(false);
    Ok(true)
}

fn _copy_sign(folder: &String, entry: &DirEntry, p: &crate::project::Project, version: &String) -> Result<bool, Error> {
    let path = entry.path();
    let cpath = path.clone();
    let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
    let pbo = cpath.replace((folder.clone() + "/").as_str(), "");
    if !path.ends_with(".pbo") && !pbo.contains(p.prefix.as_str()) {
        return Ok(false);
    }

    let modname = p.get_modname();
    fs::copy(&cpath, format!("releases/{}/@{}/{}/{}", version, modname, folder, pbo))?;

    let signame = p.get_signame(&pbo);
    let keyname = p.get_keyname();

    println!("   {} {}/{}", "Signing".green().bold(), folder, pbo);
    armake2::sign::cmd_sign(
        PathBuf::from(format!("releases/keys/{}.biprivatekey", keyname)),
        PathBuf::from(format!("releases/{}/@{}/{}/{}", version, modname, folder, pbo)),
        Some(PathBuf::from(format!("releases/{0}/@{1}/{2}/{3}", version, modname, folder, signame))),
        armake2::sign::BISignVersion::V3
    ).print_error(false);
    Ok(true)
}
